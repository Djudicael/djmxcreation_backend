import { html, nothing } from "lit-html";

export function imageSources(media) {
    return media?.optimization?.status === "ready" ? (media.optimization.sources || []).filter(source => source.url && source.width > 0) : [];
}
export function imageUrl(media, width = 1600) {
    const sources = imageSources(media).filter(source => source.mimeType !== "image/webp").sort((a,b) => a.width-b.width);
    return (sources.find(source => source.width >= width) || sources.at(-1))?.url || media?.url;
}
export function imageSrcset(media, webp = false) {
    const byWidth = new Map();
    for (const source of imageSources(media)) if ((source.mimeType === "image/webp") === webp) byWidth.set(source.width, source.url);
    // A small WebP must not replace a larger JPEG on high resolution screens.
    if (webp && Math.max(0,...byWidth.keys()) < Math.max(0,...imageSources(media).filter(source=>source.mimeType!=="image/webp").map(source=>source.width))) return "";
    return [...byWidth].sort((a,b)=>a[0]-b[0]).map(([width,url])=>`${url} ${width}w`).join(", ");
}
export function responsiveImage(media, { alt="", className="", loading="lazy", priority="auto", sizes="100vw", onLoad=()=>{} }={}) {
    const baseline=imageSrcset(media), webp=imageSrcset(media,true);
    return html`<picture>${webp ? html`<source type="image/webp" srcset=${webp} sizes=${sizes}>` : ""}<img class=${className} src=${imageUrl(media)} srcset=${baseline || nothing} sizes=${sizes} width=${media?.optimization?.width || nothing} height=${media?.optimization?.height || nothing} alt=${alt} loading=${loading} fetchpriority=${priority}
        @load=${onLoad} @error=${event => {
            const img=event.currentTarget;
            if (!media?.url) return;
            const original=new URL(media.url,img.baseURI).href;
            if (img.src===original && !img.srcset && !img.closest("picture")?.querySelector("source")) return;
            img.closest("picture")?.querySelectorAll("source").forEach(source=>source.remove());
            img.removeAttribute("srcset"); img.src=original;
        }}></picture>`;
}
