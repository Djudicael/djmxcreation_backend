import { TemplateRenderer, html, safeHTML, LoadState } from "../utils/template-renderer";
import { EventBinder } from "../../../shared/src/event-binder.js";
import portfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu?.hideMenu?.();
        this.routerOutlet = document.querySelector('router-outlet');
        this.doc = document.documentElement;
        this.api = portfolioApi;
        this.title = null;
        this.subTitle = null;
        this.client = null;
        this.contents = null;
        this.$image = null;
        this.initImageOverlayEvent = this.initImageOverlayEvent.bind(this);
        this._overlayClickHandler = () => this.hideImageOverlay();
        this._overlayKeyHandler = (e) => {
            if (e.key === "Escape") this.hideImageOverlay();
        };
        this._imageClickBinder = new EventBinder();
    }

    getTitleFragment(title) {
        if (title) {
            return html`<div class="project_title" >
            <h1> ${title}</h1>
        </div>`;
        }

        return html``;
    }

    getFragmentSubtitle(subTitle) {
        if (subTitle) {
            return html`<div class="project_subtitle">
            <h2>${subTitle}</h2>
        </div>`;
        }
        return html``;
    }

    getFragmentClient(client) {
        if (client) {
            return html`<div class="project_client">
            Client: ${client}
        </div>`;
        }
        return html``;
    }

    get template() {
        if (this.isLoading) return this.loadingTemplate;
        if (this.hasError) return this.errorTemplate;

        const description = html`${safeHTML(htmlDescription(this.description))}`;
        const projectHeader = html`
        <div class="project_header">	
            ${this.getTitleFragment(this.title)}
            ${this.getFragmentSubtitle(this.subTitle)}
            ${this.getFragmentClient(this.client)}
        </div>`;
        const imageOverlay = html`
        <div class="image-overlay" role="dialog" aria-modal="true" aria-label="Image preview">
            <div class="image-overlay-content">
            <img src="" class="overlay-image" alt="Full size preview">
            </div>
        </div>`;

        const images = this.contents
            ? html`${this.contents.map(({ url }, index) => html`
            <img
            src="${url}"
            data-hi-res="${url}"
            class="pro-image"
            alt="${this.title ? `${this.title} - image ${index + 1}` : `Project image ${index + 1}`}"
            loading="lazy"
            role="button"
            tabindex="0"
            >
    `)}` : html``;

        return html`
        <section class="projects-section">
                ${projectHeader}
            <div id="description" class="project_description">
                ${description}
            </div>
            <div class="project_content">
                ${images}
            </div>
            ${imageOverlay}
        </section>
        `;
    }

    async getProject(id) {
        this.setLoadState(LoadState.LOADING);
        this.render();
        try {
            const { metadata, description, contents } = await this.api.getProject(id);
            if (!this.isConnected) return;
            this.title = metadata.title;
            this.subTitle = metadata.subTitle;
            this.client = metadata.client;
            this.description = description;
            this.contents = contents;
            this.setLoadState(LoadState.DONE);
            super.render();
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load project.");
            if (this.isConnected) this.render();
        }
    }

    async init() {
        this.$image = this.querySelector('.image-overlay');
        this.$image?.addEventListener('click', this._overlayClickHandler);
        document.addEventListener('keydown', this._overlayKeyHandler);
        this.initImageOverlayEvent();
    }

    async connectedCallback() {
        super.connectedCallback();
        const location = await this.routerOutlet?.getLocation?.(window.location.pathname);
        if (!location?.params?.id) {
            return;
        }
        const id = location.params.id;
        await this.getProject(id);
        if (!this.isConnected) {
            return;
        }
        this.content = this.querySelector('.content');
        await this.init();

    }

    showImageOverlay(e) {
        this._lastFocusedElement = document.activeElement;
        const element = e.currentTarget;
        const url = element.dataset.hiRes;
        const overlay = this.querySelector(".image-overlay");
        if (!overlay) {
            return;
        }
        let image = overlay.querySelector(".overlay-image");
        if (!image) {
            return;
        }
        image.src = url;
        image.alt = element.alt || "Full size preview";
        overlay.style.display = "flex";
        overlay.focus();
    }

    hideImageOverlay() {
        const overlay = this.querySelector(".image-overlay");
        if (!overlay) {
            return;
        }
        overlay.style.display = "none";
        this._lastFocusedElement?.focus?.();
    }

    _handleImageKeydown(e) {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            this.showImageOverlay(e);
        }
    }

    initImageOverlayEvent = () => {
        this._imageClickBinder.bindAll(
            this.querySelectorAll('.pro-image'), 'click', (e) => this.showImageOverlay(e)
        );
        this.querySelectorAll('.pro-image').forEach(img => {
            img.addEventListener('keydown', (e) => this._handleImageKeydown(e));
        });
    }

    disconnectedCallback() {
        super.disconnectedCallback();
        this.$image?.removeEventListener('click', this._overlayClickHandler);
        document.removeEventListener('keydown', this._overlayKeyHandler);
        this._imageClickBinder.unbindAll();
    }
}