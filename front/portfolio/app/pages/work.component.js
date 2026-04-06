import { TemplateRenderer, html, sanitizeHtml, unsafeHTML } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu?.hideMenu?.();
        this.routerOutlet = document.querySelector('router-outlet');
        this.doc = document.documentElement;
        this.api = new PortfolioApi();
        this.title;
        this.subTitle;
        this.client;
        this.contents;
        this.$image;
        this.initImageOverlayEvent = this.initImageOverlayEvent.bind(this);
        this._overlayClickHandler = () => this.hideImageOverlay();
        this._imageClickHandlers = [];

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
        const safeDescription = sanitizeHtml(htmlDescription(this.description));
        const description = html`${unsafeHTML(safeDescription)}`;
        const projectHeader = html`
        <div class="project_header">	
            ${this.getTitleFragment(this.title)}
            ${this.getFragmentSubtitle(this.subTitle)}
            ${this.getFragmentClient(this.client)}
        </div>`;
        const imageOverlay = html`
        <div class="image-overlay" >
            <div class="image-overlay-content">
            <img src="" class="overlay-image">
            </div>
        </div>`;

        const images = this.contents
            ? html`${this.contents.map(({ url }) => html`
            <img
            src="${url}"
            data-hi-res="${url}"
            class="pro-image" 
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
        try {
            const { metadata, description, contents } = await this.api.getProject(id);
            this.title = metadata.title;
            this.subTitle = metadata.subTitle;
            this.client = metadata.client;
            this.description = description;
            this.contents = contents;
            if (this.isConnected) {
                super.render();
            }
        } catch (error) {
            console.error('Failed to load work', error);
        }
    }

    async init() {
        this.$image = this.querySelector('.image-overlay');
        this.$image?.addEventListener('click', this._overlayClickHandler);
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
        overlay.style.display = "flex";
    }

    hideImageOverlay() {
        const overlay = this.querySelector(".image-overlay");
        if (!overlay) {
            return;
        }
        overlay.style.display = "none";
    }

    initImageOverlayEvent = () => {
        this.querySelectorAll('.pro-image').forEach((item, index) => {
            const oldHandler = this._imageClickHandlers[index];
            if (oldHandler) {
                item.removeEventListener('click', oldHandler);
            }
        });

        this._imageClickHandlers = [];
        this.querySelectorAll('.pro-image').forEach(item => {
            const handler = (event) => this.showImageOverlay(event);
            this._imageClickHandlers.push(handler);
            item.addEventListener('click', handler)
        });
    }

    disconnectedCallback() {
        this.$image?.removeEventListener('click', this._overlayClickHandler);
        this.querySelectorAll('.pro-image').forEach((item, index) => {
            const handler = this._imageClickHandlers[index];
            if (handler) {
                item.removeEventListener('click', handler);
            }
        });
        this._imageClickHandlers = [];
    }
}