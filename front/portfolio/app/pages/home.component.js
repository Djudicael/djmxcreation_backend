import { TemplateRenderer, html, LoadState } from "../utils/template-renderer";
import portfolioApi from "../api/portfolio.api.js";

export default class HomeComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu?.hideMenu?.();
        this.api = portfolioApi;
        this.projects = [];
        this.page = 1;
        this.pageSize = 6;
        this.totalPages = 0;
    }

    get template() {
        if (this.isLoading && !this.projects.length) return this.loadingTemplate;
        if (this.hasError && !this.projects.length) return this.errorTemplate;

        const projects = this.projects.length
            ? html`${this.projects.map(({ id, metadata, thumbnail }) => html`
                <div class="gallery-item">
                    <img src="${thumbnail?.url}" alt="${metadata?.title || 'Project'}" loading="lazy">
                </div>`)}`
            : html``;

        return html`
        <section class="content-page">
        <div class="gallery" role="list" aria-label="Project gallery">
            ${projects}
        </div>
        </section>
        `;
    }

    async getProjects() {
        try {
            const { totalPages, page, size, projects } = await this.api.getProjects({ page: this.page, pageSize: this.pageSize });
            if (!this.isConnected) return;
            this.totalPages = totalPages;
            this.page = page;
            this.pageSize = size;
            this.projects.push(...projects);
            this.setLoadState(LoadState.DONE);
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load projects.");
        }
        if (this.isConnected) super.render();
    }

    async connectedCallback() {
        super.connectedCallback();
        this.setLoadState(LoadState.LOADING);
        this.render();
        await this.getProjects();
    }
}
