import { TemplateRenderer, html, safeHTML, LoadState } from "../utils/template-renderer";
import portfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";
import { mediaKind } from "../../../shared/src/media-kind.js";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        document.querySelector("c-header")?.hideMenu?.();
        this.projectTitle = "";
        this.contents = [];
        this.revealed = false;
        this.imageIndex = 0;
        this.mediaRenewals = new Map();
    }

    get images() {
        return [...this.contents.filter(content => !content.behindScenes), ...this.contents.filter(content => content.behindScenes)].filter(content => mediaKind(content) === "image");
    }
    get hasScenes() {
        return this.contents.some(content => content.behindScenes) || this.description?.behindScenes?.ops?.some(op => typeof op.insert === "string" ? op.insert.trim() : op.insert);
    }

    gallery(contents) {
        return html`<div class="project-gallery">${contents.map((content, index) => {
            const kind = mediaKind(content);
            if (kind === "video") return html`<figure class="project-media"><video src=${content.url} poster=${content.posterUrl || ""} controls playsinline preload="none" aria-label=${content.caption || `${this.projectTitle} — video ${index + 1}`} @play=${event => { if (Date.now() - this.mediaLoadedAt > 3600000) this.renewVideo(content, event.currentTarget); }} @error=${event => this.renewVideo(content, event.currentTarget)}></video>
                ${content.caption ? html`<figcaption>${content.caption}</figcaption>` : ""}
                <p class="video-status" role="status"></p>
                <button class="text-link video-retry" hidden @click=${event => this.renewVideo(content, event.currentTarget.closest("figure").querySelector("video"), true)}>Retry video</button>
            </figure>`;
            if (kind === "file") return html`<a class="action-link file-link project-file" href=${content.url}>Open project file <span aria-hidden="true">↗</span></a>`;
            const imageIndex = this.images.indexOf(content);
            return html`<figure class="project-media project-photo" data-image-index=${imageIndex}><button class="gallery-image" aria-label=${`Enlarge ${content.alt || this.projectTitle} — image ${imageIndex + 1}`} @click=${event => this.openImage(imageIndex, event.currentTarget)}><img class="pro-image" src=${content.url} alt=${content.alt || `${this.projectTitle} — image ${imageIndex + 1}`} loading=${imageIndex === 0 ? "eager" : "lazy"} fetchpriority=${imageIndex === 0 ? "high" : "auto"} @load=${() => this.arrangeGallery()}><span class="image-open" aria-hidden="true">↗</span></button><figcaption>${content.caption || String(imageIndex + 1).padStart(2, "0")}</figcaption></figure>`;
        })}</div>`;
    }

    get template() {
        if (this.isLoading) return this.loadingTemplate;
        if (this.hasError) return this.errorTemplate;
        const gated = this.adult && !this.revealed;
        return html`
            <article class="work-page page-shell">
                <a class="text-link back-link" href="/works"><span aria-hidden="true">←</span> All works</a>
                <header class="project-heading"><p class="eyebrow">Selected project ${this.adult ? "/ NSFW · 18+" : ""}</p><h1>${this.projectTitle}</h1>${this.subTitle ? html`<p class="project-subtitle">${this.subTitle}</p>` : ""}</header>
                ${gated ? html`
                    <section class="adult-notice" aria-labelledby="adult-heading"><span class="eyebrow">Content notice / 18+</span><h2 id="adult-heading">For adult audiences.</h2><p>This project contains sensitive imagery. Choose whether you would like to view it.</p><div class="page-actions"><button class="action-link action-primary" @click=${() => this.setRevealed(true)}>View project <span aria-hidden="true">↗</span></button><a class="text-link" href="/works">Back to works</a></div></section>
                ` : html`
                    <div class="project-intro">${this.client ? html`<div class="project-client"><span class="eyebrow">Client</span><p>${this.client}</p></div>` : ""}<div class="rich-text project-description">${safeHTML(htmlDescription(this.description))}</div></div>
                    ${this.adult ? html`<button class="text-link hide-adult" @click=${() => this.setRevealed(false)}>Hide adult content</button>` : ""}
                    ${this.gallery(this.contents.filter(content => !content.behindScenes))}
                    ${this.hasScenes ? html`<section class="project-scenes" aria-labelledby="scenes-title"><h2 id="scenes-title">Behind the scenes</h2><div class="rich-text scenes-description">${safeHTML(htmlDescription(this.description?.behindScenes))}</div>${this.gallery(this.contents.filter(content => content.behindScenes))}</section>` : ""}
                `}
                <footer class="project-footer"><a class="text-link" href="/works">← Back to works</a><a class="action-link" href="/contact">Contact <span aria-hidden="true">↗</span></a></footer>
                <dialog class="image-lightbox" aria-label="Image preview" @click=${event => { if (event.target === event.currentTarget) event.currentTarget.close(); }} @close=${() => { document.body.classList.remove("lightbox-open"); this.imageTrigger?.focus(); }} @keydown=${event => {
                    if (event.key === "ArrowLeft" || event.key === "ArrowRight") { event.preventDefault(); this.changeImage(event.key === "ArrowLeft" ? -1 : 1); }
                }}>
                    <div class="lightbox-toolbar"><span class="lightbox-count" aria-live="polite"></span><button class="lightbox-close" @click=${() => this.querySelector("dialog").close()}>Close <span aria-hidden="true">×</span></button></div>
                    <img class="overlay-image" alt="">
                    <div class="lightbox-navigation"><button aria-label="Previous image" @click=${() => this.changeImage(-1)}>←</button><button aria-label="Next image" @click=${() => this.changeImage(1)}>→</button></div>
                </dialog>
            </article>
        `;
    }

    arrangeGallery() {
        // Only inspect rendered images: adult media are never preloaded before consent.
        // Keep the DOM order and focused button intact when adjacent portraits pair up.
        for (const gallery of this.querySelectorAll(".project-gallery")) {
        const items = [...gallery.children];
        const isPortrait = item => {
            const image = item?.querySelector(".pro-image");
            return image?.naturalWidth > 0 && image.naturalWidth / image.naturalHeight <= 1.1;
        };
        for (const item of items) item.dataset.layout = "full";
        for (let index = 0; index < items.length; index++) {
            const item = items[index];
            if (!isPortrait(item)) continue;
            if (item.dataset.imageIndex !== "0" && isPortrait(items[index + 1])) {
                item.dataset.layout = "pair-left";
                items[++index].dataset.layout = "pair-right";
            } else {
                item.dataset.layout = "portrait";
            }
        }
        }
    }

    async renewVideo(content, video, manual = false) {
        const figure = video.closest("figure");
        if (!figure || video.dataset.renewing === "true") return;
        if (Date.now() - (this.mediaRenewals.get(content.id) || 0) < 60000 && !manual) {
            figure.querySelector(".video-status").textContent = "This video could not be played. Retry or check your browser's format support.";
            figure.querySelector(".video-retry").hidden = false;
            return;
        }
        this.mediaRenewals.set(content.id, Date.now());
        video.dataset.renewing = "true";
        const time = Number.isFinite(video.currentTime) ? video.currentTime : 0;
        const resume = !video.paused;
        video.pause();
        try {
            const project = await portfolioApi.getProject(this.projectId, { cache: false });
            if (!this.isConnected || !video.isConnected) return;
            if (project.adult && !this.revealed) { this.adult = true; this.render(); return; }
            const fresh = project.contents?.find(item => item.id === content.id);
            if (!fresh?.url) throw new Error("Video unavailable");
            content.url = fresh.url;
            video.onloadedmetadata = () => { video.currentTime = time; if (resume) video.play().catch(() => {}); };
            video.src = fresh.url;
            video.poster = fresh.posterUrl || "";
            video.load();
            this.mediaLoadedAt = Date.now();
            figure.querySelector(".video-status").textContent = "";
            figure.querySelector(".video-retry").hidden = true;
        } catch {
            if (video.isConnected) {
                video.removeAttribute("src");
                video.load();
                figure.querySelector(".video-status").textContent = "Video unavailable. Please try again.";
                figure.querySelector(".video-retry").hidden = false;
            }
        } finally { video.dataset.renewing = "false"; }
    }

    setRevealed(value) {
        this.revealed = value;
        this.render();
        // Move focus to the newly available action rather than leaving it on a removed button.
        this.querySelector(value ? ".hide-adult" : ".adult-notice button")?.focus();
    }

    openImage(index, trigger) {
        this.imageIndex = index;
        this.imageTrigger = trigger;
        this.updateImage();
        this.querySelector("dialog").showModal();
        document.body.classList.add("lightbox-open");
    }

    changeImage(direction) {
        this.imageIndex = (this.imageIndex + direction + this.images.length) % this.images.length;
        this.updateImage();
    }

    updateImage() {
        const image = this.querySelector(".overlay-image");
        image.src = this.images[this.imageIndex].url;
        image.alt = this.images[this.imageIndex].alt || `${this.projectTitle} — image ${this.imageIndex + 1}`;
        this.querySelector(".lightbox-count").textContent = `${this.imageIndex + 1} / ${this.images.length}`;
        this.querySelector(".lightbox-navigation").hidden = this.images.length < 2;
    }

    async connectedCallback() {
        super.connectedCallback();
        this.setLoadState(LoadState.LOADING);
        this.render();
        const location = await document.querySelector("router-outlet")?.getLocation?.(window.location.pathname);
        if (!location?.params?.id) return;
        this.projectId = location.params.id;
        try {
            const data = await portfolioApi.getProject(location.params.id);
            if (!this.isConnected) return;
            this.projectTitle = data.metadata?.title || "Untitled project";
            this.subTitle = data.metadata?.subTitle;
            this.client = data.metadata?.client;
            this.description = data.description;
            this.contents = data.contents || [];
            this.mediaLoadedAt = Date.now();
            this.adult = data.adult;
            this.setLoadState(LoadState.DONE);
            document.title = `${this.projectTitle} — DJMXCreation`;
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load project.");
        }
        if (this.isConnected) this.render();
    }

    disconnectedCallback() {
        document.body.classList.remove("lightbox-open");
        super.disconnectedCallback();
    }
}
