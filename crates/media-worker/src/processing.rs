use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, Limits};
use std::io::Cursor;

pub const WIDTHS: [u32; 4] = [400, 800, 1600, 2560];
pub const MAX_PIXELS: u64 = 24_000_000;
pub const MAX_OUTPUT_PIXELS: u64 = 3_500_000;

pub struct EncodedImage {
    pub mime: &'static str,
    pub extension: &'static str,
    pub bytes: Vec<u8>,
}

pub struct Variant {
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    pub baseline: EncodedImage,
    pub webp: Option<EncodedImage>,
}

/// Pure Rust, single-threaded and filesystem-free, including on wasm32-wasip2.
pub fn resize(bytes: &[u8], target: u32) -> Result<Variant, String> {
    if bytes.len() > 32 * 1024 * 1024 || !WIDTHS.contains(&target) {
        return Err("Image input exceeds processing limits".into());
    }
    let format = image::guess_format(bytes).map_err(|_| "Unrecognized image")?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(12000);
    limits.max_image_height = Some(12000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    match format {
        ImageFormat::Png => {
            let decoder =
                image::codecs::png::PngDecoder::with_limits(Cursor::new(bytes), limits.clone())
                    .map_err(|_| "Invalid PNG")?;
            if decoder.is_apng().map_err(|_| "Invalid PNG animation")? {
                return Err("Animated images keep their original".into());
            }
            if decoder
                .gamma_value()
                .map_err(|_| "Invalid PNG gamma")?
                .is_some_and(|gamma| (gamma - 0.45455).abs() > 0.0001)
            {
                return Err("Custom PNG gamma keeps its original colors".into());
            }
        }
        ImageFormat::WebP => {
            let decoder = image::codecs::webp::WebPDecoder::new(Cursor::new(bytes))
                .map_err(|_| "Invalid WebP")?;
            if decoder.has_animation() {
                return Err("Animated images keep their original".into());
            }
        }
        ImageFormat::Jpeg => {}
        _ => {
            return Err(
                "This format keeps its original (supported: static JPEG, PNG, WebP)".into(),
            );
        }
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|_| "Image decoding failed")?;
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err("Images above 24 megapixels keep their original".into());
    }
    let profile = decoder.icc_profile().map_err(|_| "Invalid color profile")?;
    if profile
        .as_ref()
        .is_some_and(|icc| icc.len() < 20 || &icc[16..20] != b"RGB ")
    {
        return Err("Unsupported color profile; keeping original colors".into());
    }
    let orientation = decoder
        .orientation()
        .map_err(|_| "Invalid image orientation")?;
    let mut decoded = DynamicImage::from_decoder(decoder).map_err(|_| "Image decoding failed")?;
    decoded.apply_orientation(orientation);
    let original_width = decoded.width();
    let original_height = decoded.height();
    let pixel_scale = (MAX_OUTPUT_PIXELS as f64
        / (f64::from(original_width) * f64::from(original_height)))
    .sqrt()
    .min(1.0);
    let width = target
        .min(original_width)
        .min((f64::from(original_width) * pixel_scale).floor().max(1.0) as u32);
    let height =
        (u64::from(original_height) * u64::from(width) / u64::from(original_width)).max(1) as u32;
    let resized = if width < original_width {
        // Integer convolution keeps photographic quality within WASI fuel limits.
        let source = if decoded.color().has_alpha() {
            DynamicImage::ImageRgba8(decoded.into_rgba8())
        } else {
            DynamicImage::ImageRgb8(decoded.into_rgb8())
        };
        let mut destination = if source.color().has_alpha() {
            DynamicImage::ImageRgba8(image::RgbaImage::new(width, height))
        } else {
            DynamicImage::ImageRgb8(image::RgbImage::new(width, height))
        };
        fast_image_resize::Resizer::new()
            .resize(
                &source,
                &mut destination,
                &fast_image_resize::ResizeOptions::new().resize_alg(
                    fast_image_resize::ResizeAlg::Convolution(
                        fast_image_resize::FilterType::Lanczos3,
                    ),
                ),
            )
            .map_err(|_| "Image resizing failed")?;
        destination
    } else {
        decoded
    };
    let width = resized.width();
    let height = resized.height();
    let mut baseline = Vec::new();
    let (baseline_mime, baseline_extension) = if resized.color().has_alpha() {
        let pixels = resized.to_rgba8();
        let mut encoder = image::codecs::png::PngEncoder::new(&mut baseline);
        if let Some(icc) = &profile {
            encoder
                .set_icc_profile(icc.clone())
                .map_err(|_| "Color profile cannot be preserved")?;
        }
        encoder
            .write_image(
                pixels.as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|_| "PNG encoding failed")?;
        ("image/png", "png")
    } else {
        let pixels = resized.to_rgb8();
        let mut encoder = jpeg_encoder::Encoder::new(&mut baseline, 85);
        if let Some(icc) = &profile {
            encoder
                .add_icc_profile(icc)
                .map_err(|_| "Color profile cannot be preserved")?;
        }
        encoder
            .encode(
                pixels.as_raw(),
                width as u16,
                height as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|_| "JPEG encoding failed")?;
        ("image/jpeg", "jpg")
    };
    let webp = if u64::from(width) * u64::from(height) <= 2_000_000 {
        let pixels = resized.to_rgba8();
        let mut webp = Vec::new();
        let mut encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut webp);
        if let Some(icc) = &profile {
            encoder
                .set_icc_profile(icc.clone())
                .map_err(|_| "Color profile cannot be preserved")?;
        }
        encoder
            .write_image(
                pixels.as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|_| "WebP encoding failed")?;
        (webp.len() < baseline.len()).then_some(EncodedImage {
            mime: "image/webp",
            extension: "webp",
            bytes: webp,
        })
    } else {
        None
    };
    Ok(Variant {
        width,
        height,
        original_width,
        original_height,
        baseline: EncodedImage {
            mime: baseline_mime,
            extension: baseline_extension,
            bytes: baseline,
        },
        webp,
    })
}
