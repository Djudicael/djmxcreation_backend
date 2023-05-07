import { TemplateRenderer, html, unsafeHTML } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
        this.routerOutlet = document.querySelector('router-outlet');
        this.doc = document.documentElement;
        this.api = new PortfolioApi();
        this.title;
        this.subTitle;
        this.client;
        this.contents;
        this.$image;
        this.initImageOverlayEvent = this.initImageOverlayEvent.bind(this);

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
        const description = html`${unsafeHTML(htmlDescription(this.description))}`;
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
        const { metadata, description, contents } = await this.api.getProject(id);
        this.title = metadata.title;
        this.subTitle = metadata.subTitle;
        this.client = metadata.client;
        this.description = description;
        this.contents = contents;
        super.render();
    }

    async init() {
        this.$image = this.querySelector('.image-overlay');
        this.$image.addEventListener('click', _ => this.hideImageOverlay());
        this.initImageOverlayEvent();

    }

    async connectedCallback() {
        super.connectedCallback();
        const location = await this.routerOutlet.getLocation(window.location.pathname);
        const id = location.params.id;
        await this.getProject(id);
        this.content = this.querySelector('.content');
        await this.init();

    }

    showImageOverlay(e) {
        const element = e.currentTarget;
        const url = element.dataset.hiRes;
        const overlay = this.querySelector(".image-overlay");
        let image = overlay.querySelector(".overlay-image");
        image.src = url;
        overlay.style.display = "flex";
    }

    hideImageOverlay() {
        const overlay = this.querySelector(".image-overlay");
        overlay.style.display = "none";
    }

    initImageOverlayEvent = () => {
        this.querySelectorAll('.pro-image').forEach(item => {
            item.addEventListener('click', e => this.showImageOverlay(e))
        });
    }
}