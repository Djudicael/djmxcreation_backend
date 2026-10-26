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
    }

    get images() { return this.contents.filter(content => mediaKind(content) === "image"); }

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
                    <div class="project-gallery">${this.contents.map((content, index) => {
                        const kind = mediaKind(content);
                        if (kind === "video") return html`<figure><video src=${content.url} controls preload="metadata" aria-label=${`${this.projectTitle} — video ${index + 1}`}></video></figure>`;
                        if (kind === "file") return html`<a class="action-link file-link" href=${content.url}>Open project file <span aria-hidden="true">↗</span></a>`;
                        const imageIndex = this.images.indexOf(content);
                        return html`<figure><button class="gallery-image" aria-label=${`Enlarge ${this.projectTitle} — image ${imageIndex + 1}`} @click=${event => this.openImage(imageIndex, event.currentTarget)}><img class="pro-image" src=${content.url} alt=${`${this.projectTitle} — image ${imageIndex + 1}`} loading="lazy"><span class="image-open" aria-hidden="true">↗</span></button><figcaption class="eyebrow">Plate ${String(imageIndex + 1).padStart(2, "0")}</figcaption></figure>`;
                    })}</div>
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
        image.alt = `${this.projectTitle} — image ${this.imageIndex + 1}`;
        this.querySelector(".lightbox-count").textContent = `${this.imageIndex + 1} / ${this.images.length}`;
        this.querySelector(".lightbox-navigation").hidden = this.images.length < 2;
    }

    async connectedCallback() {
        super.connectedCallback();
        this.setLoadState(LoadState.LOADING);
        this.render();
        const location = await document.querySelector("router-outlet")?.getLocation?.(window.location.pathname);
        if (!location?.params?.id) return;
        try {
            const data = await portfolioApi.getProject(location.params.id);
            if (!this.isConnected) return;
            this.projectTitle = data.metadata?.title || "Untitled project";
            this.subTitle = data.metadata?.subTitle;
            this.client = data.metadata?.client;
            this.description = data.description;
            this.contents = data.contents || [];
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
