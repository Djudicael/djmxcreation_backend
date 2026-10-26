import Quill from "quill";
import { render } from "lit-html";
import { html } from "../utils/template-renderer.js";
import { AdminPage } from "../utils/admin-page.js";
import portfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";
import { mediaKind } from "../../../shared/src/media-kind.js";

export class ProjectComponent extends AdminPage {
    constructor() {
        super();
        this.project = null;
        this.projectId = decodeURIComponent(window.location.pathname.split("/")[2] || "");
    }

    get template() {
        if (!this.project) return html`<section class="content-page admin-page"><p role="status">Loading project…</p></section>`;
        const p = this.project;
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><a class="admin-back" href="/projects">← All projects</a>
                <h1 data-project-title>${p.metadata?.title}</h1><p>Save your changes before leaving this page.</p></div>
            </header>${this.statusTemplate}
            <form class="admin-panel" @submit=${event => this.save(event)}>
                <h2>Project details</h2><div class="admin-fields">
                    <label>Title<input name="title" required maxlength="200" .value=${p.metadata?.title || ""} /></label>
                    <label>Subtitle<input name="subTitle" maxlength="200" .value=${p.metadata?.subTitle || ""} /></label>
                    <label>Client<input name="client" maxlength="200" .value=${p.metadata?.client || ""} /></label>
                </div>
                <label class="admin-editor-label">Description</label><div data-editor></div>
                <div class="admin-options">
                    <label><input name="visible" type="checkbox" .checked=${p.visible} /> Publish on portfolio</label>
                    <label><input name="adult" type="checkbox" .checked=${p.adult} /> Adult content</label>
                </div>
                <div class="admin-actions"><button class="admin-button" type="submit">Save project</button></div>
            </form>
            <section class="admin-panel"><h2>Media and cover</h2><p>Images or videos, up to 32 MiB per file. Choose an image as the cover. Your text edits are preserved during these operations.</p>
                <form class="admin-upload" @submit=${event => this.upload(event)}>
                    <label>Files to upload<input name="files" type="file" accept="image/*,video/*" multiple required /></label>
                    <button class="admin-button secondary" type="submit">Upload files</button>
                </form><div data-media></div>
            </section>
        </section>`;
    }

    renderMedia() {
        const target = this.querySelector("[data-media]");
        if (!target) return;
        render(html`
            ${this.project.thumbnail?.url ? html`<div class="admin-current-cover"><img src=${this.project.thumbnail.url} alt="Current cover" /><span>Current cover</span></div>` : html`<p class="admin-muted">No cover selected.</p>`}
            <div class="admin-media-grid">${(this.project.contents || []).map(content => html`
                <article class="admin-media-card" data-content-id=${content.id}>
                    ${mediaKind(content) === "video" ? html`<video src=${content.url} controls preload="metadata"></video>` :
                      mediaKind(content) === "image" ? html`<img src=${content.url} alt="Project media" />` :
                      html`<a href=${content.url} target="_blank" rel="noopener noreferrer">Open file</a>`}
                    <div class="admin-actions">
                        ${mediaKind(content) === "image" ? html`<button class="admin-button secondary" @click=${() => this.cover(content)}>Set as cover</button>` : ""}
                        <button class="admin-button danger" @click=${() => this.remove(content)}>Delete media</button>
                    </div>
                </article>`)}</div>
        `, target);
    }

    async save(event) {
        event.preventDefault();
        const data = new FormData(event.currentTarget);
        const title = data.get("title").trim();
        if (!title) { this.feedback("A title is required.", true); return; }
        await this.perform("Project saved.", async () => {
            await portfolioApi.updateProject(this.projectId, {
                metadata: { title, subTitle: data.get("subTitle").trim(), client: data.get("client").trim() },
                description: this.editor.getContents(),
                visible: data.has("visible"),
                adult: data.has("adult"),
            });
            this.querySelector("[data-project-title]").textContent = title;
        });
    }

    async refreshMedia() {
        this.project = await portfolioApi.getProject(this.projectId);
        if (this.isConnected) this.renderMedia();
    }

    async upload(event) {
        event.preventDefault();
        const input = event.currentTarget.querySelector("[name=files]");
        const files = [...input.files];
        if (!files.length) return;
        await this.perform("Files uploaded.", async () => {
            for (const file of files) {
                if (file.size > 32 * 1024 * 1024) throw new Error(`${file.name} exceeds the 32 MiB limit.`);
                await portfolioApi.addContentToProject(this.projectId, { file });
            }
            await this.refreshMedia();
            input.value = "";
        });
    }

    async cover(content) {
        await this.perform("Cover updated.", async () => {
            await portfolioApi.addThumbnail(this.projectId, content.id);
            await this.refreshMedia();
        });
    }

    async remove(content) {
        if (!window.confirm("Permanently delete this media?")) return;
        await this.perform("Media deleted.", async () => {
            await portfolioApi.deleteContent(this.projectId, content.id);
            await this.refreshMedia();
        });
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.load(async () => { this.project = await portfolioApi.getProject(this.projectId); });
        if (!this.isConnected || !this.project) return;
        this.editor = new Quill(this.querySelector("[data-editor]"), editorConfig);
        this.editor.root.setAttribute("aria-label", "Project description");
        if (this.project.description?.ops) this.editor.setContents(this.project.description);
        this.renderMedia();
    }

    disconnectedCallback() {
        this.editor?.disable();
        super.disconnectedCallback();
    }
}
