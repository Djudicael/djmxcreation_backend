import { TemplateRenderer, html } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";

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

    get template() {

        const projectHeader = html`
        <div class="project_header">				
            <div class="project_title" >
                <h1> ${this.title}</h1>
            </div>
            <div class="project_subtitle">
                <h2>${this.subTitle}</h2>
            </div>
            <div class="project_client">
                Client: ${this.client}
            </div>
        </div>`;

        const imageOverlay = html`
      <div class="image-overlay" >
        <div class="image-overlay-content">
          <img src="" class="overlay-image">
        </div>
      </div>
    `;

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

    init() {
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
        this.init();

    }

    showImageOverlay(e) {
        const element = e.currentTarget;
        const url = element.dataset.hiRes;
        console.log(url);
        const overlay = this.querySelector(".image-overlay");
        let image = overlay.querySelector(".overlay-image");
        console.log(image);
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