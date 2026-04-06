import { QuillDeltaToHtmlConverter } from 'quill-delta-to-html';
export const secureImageSrc = src => src ? src.replace(/".*/g, '') : src

export const htmlDescription = description => {
    if (description?.ops && Array.isArray(description.ops)) {
        const converter = new QuillDeltaToHtmlConverter(description.ops, {});
        return converter.convert();
    }
    return '';
}

export function lerp(start, end, t) {
    return start * (1 - t) + end * t;
}