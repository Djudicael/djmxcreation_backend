// Keep completed parts in memory so Retry only sends the failed/remaining parts.
// Storage credentials and application bearer tokens never go to the signed URL.
export class DirectUpload {
    constructor(api, projectId, file) {
        this.api = api;
        this.projectId = projectId;
        this.file = file;
        this.session = null;
        this.nextPart = 1;
        this.cancelled = false;
    }

    async send(onProgress) {
        const video = ["video/mp4", "video/webm"].includes(this.file.type);
        const image = ["image/jpeg", "image/png", "image/webp", "image/avif", "image/gif"].includes(this.file.type);
        if (!video && !image) throw new Error("Use MP4/WebM video or JPEG/PNG/WebP/AVIF/GIF images.");
        if (!this.file.size || this.file.size > (video ? 2 * 1024 ** 3 : 32 * 1024 ** 2)) throw new Error("Maximum file size: 2 GiB for video, 32 MiB for images.");
        if (!this.session) this.session = await this.api.beginUpload(this.projectId, {
            fileName: this.file.name, mimeType: this.file.type, size: this.file.size,
        });
        if (this.cancelled) { await this.abort(); throw new Error("Upload cancelled."); }
        const { id, partSize, partCount } = this.session;
        while (this.nextPart <= partCount) {
            const start = (this.nextPart - 1) * partSize;
            const blob = this.file.slice(start, Math.min(start + partSize, this.file.size));
            let lastError;
            for (let attempt = 0; attempt < 3; attempt++) {
                if (this.cancelled) throw new Error("Upload cancelled.");
                try {
                    const { url } = await this.api.uploadPartUrl(this.projectId, id, this.nextPart);
                    if (this.cancelled) throw new Error("Upload cancelled.");
                    await this.put(url, blob, loaded => onProgress(start + loaded, this.file.size));
                    lastError = null;
                    break;
                } catch (error) {
                    lastError = error;
                    if (this.cancelled) throw error;
                    if (attempt < 2) await new Promise(resolve => setTimeout(resolve, 250 * (attempt + 1)));
                }
            }
            if (lastError) throw new Error("The transfer failed. Retry to continue from this part.");
            this.nextPart++;
        }
        if (this.cancelled) throw new Error("Upload cancelled.");
        return this.api.completeUpload(this.projectId, id);
    }

    put(url, blob, progress) {
        return new Promise((resolve, reject) => {
            const xhr = new XMLHttpRequest();
            this.xhr = xhr;
            xhr.open("PUT", url);
            xhr.timeout = 120000;
            xhr.upload.onprogress = event => progress(event.loaded);
            xhr.onload = () => xhr.status >= 200 && xhr.status < 300 ? resolve() : reject(new Error("Storage rejected the upload."));
            xhr.onerror = xhr.ontimeout = () => reject(new Error("Storage connection failed."));
            xhr.onabort = () => reject(new Error("Upload cancelled."));
            xhr.send(blob);
        });
    }

    async abort() {
        this.cancelled = true;
        this.xhr?.abort();
        if (this.session) await this.api.abortUpload(this.projectId, this.session.id);
    }
}
