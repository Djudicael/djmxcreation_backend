/** Older uploads have no mimeType; infer presentation from the URL path only. */
export function mediaKind(content) {
    if (content.mimeType?.startsWith("image/")) return "image";
    if (content.mimeType?.startsWith("video/")) return "video";
    try {
        const extension = new URL(content.url, "https://media.invalid").pathname.split(".").pop().toLowerCase();
        if (["png", "jpg", "jpeg", "gif", "webp", "avif", "svg", "bmp"].includes(extension)) return "image";
        if (["mp4", "webm", "mov", "m4v", "ogv"].includes(extension)) return "video";
    } catch { /* An unknown resource is displayed as a file link. */ }
    return "file";
}
