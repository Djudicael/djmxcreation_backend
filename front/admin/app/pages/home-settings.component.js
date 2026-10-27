import { imageUrl } from "../../../shared/src/responsive-image.js";
import { html } from "../utils/template-renderer.js";
import { AdminPage } from "../utils/admin-page.js";
import portfolioApi from "../api/portfolio.api.js";
import { mediaKind } from "../../../shared/src/media-kind.js";

export class HomeSettingsComponent extends AdminPage {
    constructor() {
        super();
        this.settings = null;
        this.projects = [];
        this.search = "";
        this.resultLimit = 12;
        this.selectedId = null;
        this.introduction = "";
    }

    get eligibleProjects() {
        return this.projects.filter(project => project.visible && !project.adult && project.thumbnail?.url && mediaKind(project.thumbnail) === "image");
    }

    get template() {
        if (!this.settings) return html`<section class="content-page admin-page"><p role="status">Loading home settings…</p></section>`;
        const eligible = this.eligibleProjects;
        const selected = this.selectedId;
        const selectedProject = eligible.find(project => project.id === selected);
        const normalize = value => value.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
        const query = normalize(this.search.trim());
        const matches = eligible.filter(project => normalize(`${project.metadata?.title || ""} ${project.metadata?.client || ""}`).includes(query));
        const unavailable = selected && !eligible.some(project => project.id === selected);
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><p class="admin-eyebrow">Portfolio</p><h1>Home</h1><p>Edit the introduction and featured project on your portfolio's home page.</p></div></header>
            ${this.statusTemplate}
            <form class="admin-panel" @submit=${event => this.save(event)}>
                <h2>Introduction</h2>
                <label>Short introduction<textarea name="introduction" rows="4" maxlength="500" aria-describedby="introduction-help" .value=${this.introduction} @input=${event => { this.introduction = event.target.value; }}></textarea></label>
                <p id="introduction-help" class="admin-muted">Up to 500 characters. Leave blank to hide the introduction.</p>
                <h2 class="admin-home-section">Featured project</h2>
                <p id="featured-help" class="admin-muted">Choose a published project with an image cover. Adult projects are excluded. Changes apply when you save.</p>
                <div class="home-project-selection" aria-live="polite">
                    ${selectedProject ? html`<img src=${imageUrl(selectedProject.thumbnail,400)} alt=""><div><span class="admin-muted">Selected project</span><strong>${selectedProject.metadata?.title || "Untitled project"}</strong><span>${selectedProject.metadata?.client || ""}</span></div>` : html`<div><strong>${unavailable ? "Selected project unavailable" : "No featured project"}</strong><span class="admin-muted">${unavailable ? "Choose another project or remove the selection." : "Home will display the abstract visual."}</span></div>`}
                    ${selected ? html`<button type="button" class="admin-button secondary" @click=${() => this.selectProject(null)}>Remove selection</button>` : ""}
                </div>
                <label for="home-project-search">Search projects</label>
                <input id="home-project-search" type="search" placeholder="Search by title or client" aria-describedby="featured-help" .value=${this.search} @input=${event => { this.search = event.target.value; this.resultLimit = 12; this.render(); }} @keydown=${event => { if (event.key === "Enter") event.preventDefault(); }}>
                <p class="admin-muted" role="status">${matches.length ? `Showing ${Math.min(this.resultLimit, matches.length)} of ${matches.length} projects` : "No matching projects."}</p>
                <div class="home-project-results" role="group" aria-label="Choose a featured project">
                    ${matches.slice(0, this.resultLimit).map(project => html`<button type="button" class="home-project-option" aria-pressed=${selected === project.id ? "true" : "false"} @click=${() => this.selectProject(project.id)}>
                        <img src=${imageUrl(project.thumbnail,400)} alt="" loading="lazy">
                        <span><strong>${project.metadata?.title || "Untitled project"}</strong><span>${project.metadata?.client || ""}</span><span class="home-project-choice">${selected === project.id ? "Selected" : "Select project"}</span></span>
                    </button>`)}
                </div>
                ${matches.length > this.resultLimit ? html`<button type="button" class="admin-button secondary" @click=${() => { this.resultLimit += 12; this.render(); }}>Show more projects</button>` : ""}
                ${unavailable ? html`<p class="admin-muted" role="status">The selected project is no longer available on Home. Choose another project or remove the selection before saving.</p>` : ""}
                ${!eligible.length ? html`<p class="admin-muted">No eligible projects yet. <a class="admin-text-link" href="/projects">Publish a project and set its cover.</a></p>` : ""}
                <div class="admin-actions"><button class="admin-button" type="submit">Save home</button></div>
            </form>
        </section>`;
    }

    selectProject(id) {
        this.selectedId = id;
        this.render();
        if (!id) this.querySelector("#home-project-search")?.focus();
    }

    async save(event) {
        event.preventDefault();
        const data = new FormData(event.currentTarget);
        const featuredProjectId = this.selectedId;
        await this.perform("Home saved.", async () => {
            if (featuredProjectId && !this.eligibleProjects.some(project => project.id === featuredProjectId)) {
                throw new Error("Choose an available project or remove the selection.");
            }
            this.settings = await portfolioApi.saveHomeSettings({
                introduction: data.get("introduction").trim(),
                featuredProjectId,
            });
            if (this.isConnected) this.render();
        });
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.load(async () => {
            const [settings, projects] = await Promise.all([portfolioApi.getHomeSettings(), portfolioApi.getProjects({ fresh: true })]);
            this.settings = settings;
            this.selectedId = settings.featuredProjectId;
            this.introduction = settings.introduction;
            this.projects = projects;
        });
    }
}
