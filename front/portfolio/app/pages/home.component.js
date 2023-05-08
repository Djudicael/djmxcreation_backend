import { TemplateRenderer, html } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";

export default class HomeComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
        this.api = new PortfolioApi();
        this.projects = [];
        this.page = 1;
        this.pageSize = 6;
        this.totalPages = 0;

    }

    get template() {
        const projects = this.projects ? html`${this.projects.map(({ id, metadata, adult, thumbnail }) => html`
        <div class="gallery-item">
        <img src="${thumbnail.url}" alt="${metadata.title}">
      </div>`)}` : html``;

        return html`
        <section class="content-page">
        <div class="gallery">
            ${projects}
        </div>
        </section>
        `;
    }

    async getProjects() {
        const { totalPages, page, size, projects } = await this.api.getProjects({ page: this.page, pageSize: this.pageSize });
        this.totalPages = totalPages;
        this.page = page;
        this.pageSize = size;
        this.projects.push(...projects);
        super.render();
    }

    init() {


    }


    async connectedCallback() {
        super.connectedCallback();
        await this.getProjects();

    }
}