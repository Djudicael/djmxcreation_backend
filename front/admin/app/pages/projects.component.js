import { html } from "../utils/template-renderer.js";
import { AdminPage } from "../utils/admin-page.js";
import portfolioApi from "../api/portfolio.api.js";
import { Router } from "../../../shared/src/router.js";

export class ProjectManagementComponent extends AdminPage {
    constructor() {
        super();
        this.projects = null;
        this.creating = false;
    }

    get template() {
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><p class="admin-eyebrow">Portfolio</p><h1>Projects</h1>
                <p>Create projects, add media, and choose what appears on your portfolio.</p></div>
                <button class="admin-button" @click=${() => { this.creating = true; this.render(); this.querySelector("[name=title]")?.focus(); }}>+ New project</button>
            </header>
            ${this.statusTemplate}
            ${this.creating ? html`<form class="admin-panel admin-create-form" @submit=${event => this.create(event)}>
                <h2>New project</h2><p>Your project will be created as a draft.</p>
                <div class="admin-fields">
                    <label>Title<input name="title" required maxlength="200" placeholder="Project name" /></label>
                    <label>Subtitle<input name="subTitle" maxlength="200" /></label>
                    <label>Client<input name="client" maxlength="200" /></label>
                </div><div class="admin-actions">
                    <button type="submit" class="admin-button">Create project</button>
                    <button type="button" class="admin-button secondary" @click=${() => { this.creating = false; this.render(); }}>Cancel</button>
                </div></form>` : ""}
            ${this.projects === null ? html`<p role="status">Loading projects…</p>` : this.projects.length ? html`
                <div class="admin-project-grid">${this.projects.map(project => html`
                    <article class="admin-project-card" data-project-id=${project.id}>
                        ${project.thumbnail?.url ? html`<img class="admin-cover" src=${project.thumbnail.url} alt="" />` : html`<div class="admin-cover placeholder" aria-hidden="true">DJMX</div>`}
                        <div class="admin-card-body"><span class="admin-badge">${project.visible ? "Published" : "Draft"}</span>
                            <h2>${project.metadata?.title || "Untitled"}</h2><p>${project.metadata?.subTitle || ""}</p>
                            <p class="admin-muted">${project.metadata?.client || "No client specified"}</p>
                            <div class="admin-actions"><a class="admin-button secondary" href=${`/projects/${project.id}`}>Edit</a>
                                <button class="admin-button danger" @click=${() => this.remove(project)}>Delete</button></div>
                        </div>
                    </article>`)}</div>` : html`<div class="admin-panel admin-empty"><h2>Your first project starts here</h2><p>Choose New project to create a draft, then add a description and media.</p></div>`}
        </section>`;
    }

    async create(event) {
        event.preventDefault();
        const data = Object.fromEntries(new FormData(event.currentTarget));
        data.title = data.title.trim();
        if (!data.title) { this.feedback("A title is required.", true); return; }
        await this.perform("Project created.", async () => {
            const project = await portfolioApi.createProject(data);
            if (this.isConnected) Router.go(`/projects/${project.id}`);
        });
    }

    async remove(project) {
        if (!window.confirm(`Permanently delete “${project.metadata?.title || "this project"}” and its media?`)) return;
        await this.perform("Project deleted.", async () => {
            await portfolioApi.deleteProject(project.id);
            this.projects = await portfolioApi.getProjects();
            if (this.isConnected) this.render();
        });
    }

    connectedCallback() {
        super.connectedCallback();
        this.load(async () => { this.projects = await portfolioApi.getProjects(); });
    }
}
