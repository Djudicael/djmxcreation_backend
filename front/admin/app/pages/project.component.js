import Quill from "quill";
import { render } from "lit-html";
import { repeat } from "lit-html/directives/repeat.js";
import { html } from "../utils/template-renderer.js";
import { AdminPage } from "../utils/admin-page.js";
import portfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";
import { mediaKind } from "../../../shared/src/media-kind.js";
import { DirectUpload } from "../utils/direct-upload.js";

export class ProjectComponent extends AdminPage {
    constructor() {
        super();
        this.project = null;
        this.uploadQueue = [];
        this.projectId = decodeURIComponent(window.location.pathname.split("/")[2] || "");
    }

    get template() {
        if (!this.project) return html`<section class="content-page admin-page"><p role="status">Loading project…</p></section>`;
        const p = this.project;
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><a class="admin-back" href="/projects">← All projects</a>
                <h1 data-project-title>${p.metadata?.title}</h1><p>Save your changes before leaving this page.</p></div>
                <div><a class="admin-button secondary" href=${`/preview/${this.projectId}`} target="_blank" rel="noopener" aria-describedby="preview-help">Preview project ↗</a><p id="preview-help" class="admin-muted">Preview shows saved changes only.</p></div>
            </header>${this.statusTemplate}
            <form class="admin-panel" @submit=${event => this.save(event)}>
                <h2>Project details</h2><div class="admin-fields">
                    <label>Title<input name="title" required maxlength="200" .value=${p.metadata?.title || ""} /></label>
                    <label>Subtitle<input name="subTitle" maxlength="200" .value=${p.metadata?.subTitle || ""} /></label>
                    <label>Client<input name="client" maxlength="200" .value=${p.metadata?.client || ""} /></label>
                </div>
                <label class="admin-editor-label">Description</label><div data-editor></div>
                <h2 class="admin-section-title">Behind the scenes</h2>
                <p class="admin-muted">Optional introduction to the making of this project. Assign photos and videos to this section below.</p>
                <label class="admin-editor-label">Behind the scenes description</label><div data-scenes-editor></div>
                <div class="admin-options">
                    <label><input name="visible" type="checkbox" .checked=${p.visible} /> Publish on portfolio</label>
                    <label><input name="adult" type="checkbox" .checked=${p.adult} /> Adult content</label>
                </div>
                <div class="admin-actions"><button class="admin-button" type="submit">Save project</button></div>
            </form>
            <section class="admin-panel"><h2>Media and cover</h2><p>Images up to 32 MiB; MP4/WebM videos up to 2 GiB. Videos must already be prepared for web playback. Files are sent directly to storage. Choose an image as the cover.</p>
                <form class="admin-upload" @submit=${event => this.upload(event)}>
                    <label>Files to upload<input name="files" type="file" accept="image/jpeg,image/png,image/webp,image/avif,image/gif,video/mp4,video/webm" multiple required /></label>
                    <button class="admin-button secondary" type="submit">Upload files</button>
                </form><div data-upload-progress aria-live="polite"></div><div data-media></div>
            </section>
        </section>`;
    }

    renderMedia() {
        const target = this.querySelector("[data-media]");
        if (!target) return;
        render(html`
            ${this.project.thumbnail?.url ? html`<div class="admin-current-cover"><img src=${this.project.thumbnail.url} alt="Current cover" /><span>Current cover</span></div>` : html`<p class="admin-muted">No cover selected.</p>`}
            <p class="admin-muted">Drag a media handle or use Move up / Move down to arrange its saved section. Moves are saved immediately. Save captions and section changes with Save media details.</p>
            <button type="button" class="admin-button secondary" @click=${() => this.perform("Media reloaded.", () => this.refreshMedia())}>Reload media</button>
            <div class="admin-media-grid">${repeat(this.project.contents || [], content => content.id, (content, index) => html`
                <article class="admin-media-card" data-content-id=${content.id} @dragover=${event => this.dragOver(event, content)} @dragleave=${event => { if (!event.currentTarget.contains(event.relatedTarget)) event.currentTarget.classList.remove("is-drop-target"); }} @drop=${event => this.dropMedia(event, content)}>
                    <div class="admin-media-reorder"><span>${content.behindScenes ? "Behind the scenes" : "Main gallery"}</span>
                        <button type="button" class="admin-drag-handle" draggable="true" data-move="handle" aria-label=${`Drag to reorder media ${index + 1}`} title="Drag to reorder; use the arrow buttons with a keyboard" @dragstart=${event => this.startDrag(event, content)} @dragend=${() => this.endDrag()}>⠿</button>
                        <button type="button" class="admin-move-button" data-move="up" aria-label=${`Move up media ${index + 1}`} @click=${() => this.moveMedia(content.id, -1)}>↑</button>
                        <button type="button" class="admin-move-button" data-move="down" aria-label=${`Move down media ${index + 1}`} @click=${() => this.moveMedia(content.id, 1)}>↓</button>
                    </div>
                    ${mediaKind(content) === "video" ? html`<video src=${content.url} poster=${content.posterUrl || ""} controls playsinline preload="none"></video>` :
                      mediaKind(content) === "image" ? html`<img src=${content.url} alt="Project media" />` :
                      html`<a href=${content.url} target="_blank" rel="noopener noreferrer">Open file</a>`}
                    <form class="admin-media-details" @submit=${event => this.saveMedia(event, content)}>
                        <label>Caption<textarea name="caption" rows="2" maxlength="2000">${content.caption || ""}</textarea></label>
                        ${mediaKind(content) === "image" ? html`<label>Alternative text<input name="alt" maxlength="1000" .value=${content.alt || ""}></label>` : ""}
                        <label for=${`media-section-${content.id}`}>Section</label><select id=${`media-section-${content.id}`} name="section"><option value="main" ?selected=${!content.behindScenes}>Main gallery</option><option value="scenes" ?selected=${content.behindScenes}>Behind the scenes</option></select>
                        <input type="hidden" name="position" .value=${String(content.position || 0)}>
                        ${mediaKind(content) === "video" ? html`<label for=${`media-poster-${content.id}`}>Video poster</label><select id=${`media-poster-${content.id}`} name="posterId"><option value="">None</option>${(this.project.contents || []).filter(item => mediaKind(item) === "image").map(image => html`<option value=${image.id} ?selected=${image.id === content.posterId}>${image.caption || image.alt || `Image ${(this.project.contents || []).indexOf(image) + 1}`}</option>`)}</select>` : ""}
                        <button type="submit" class="admin-button secondary">Save media details</button>
                    </form>
                    <div class="admin-actions">
                        ${mediaKind(content) === "image" ? html`<button class="admin-button secondary" @click=${() => this.cover(content)}>Set as cover</button>` : ""}
                        <button class="admin-button danger" @click=${() => this.remove(content)}>Delete media</button>
                    </div>
                </article>`)}</div>
        `, target);
        this.syncMoveButtons();
    }

    syncMoveButtons() {
        for (const content of this.project?.contents || []) {
            const siblings = this.project.contents.filter(item => Boolean(item.behindScenes) === Boolean(content.behindScenes));
            const index = siblings.findIndex(item => item.id === content.id);
            const card = this.querySelector(`[data-content-id="${content.id}"]`);
            if (!card) continue;
            card.querySelector('[data-move="up"]').disabled = this.busy || index === 0;
            card.querySelector('[data-move="down"]').disabled = this.busy || index === siblings.length - 1;
        }
    }

    async perform(message, operation) {
        const result = await super.perform(message, operation);
        this.syncMoveButtons();
        return result;
    }

    async moveMedia(id, direction, targetId) {
        if (this.busy) return;
        const contents = this.project.contents;
        const source = contents.find(item => item.id === id);
        const siblings = contents.filter(item => Boolean(item.behindScenes) === Boolean(source?.behindScenes));
        const destination = targetId || siblings[siblings.indexOf(source) + direction]?.id;
        if (!source || !destination || destination === id || !siblings.some(item => item.id === destination)) return;
        const previous = contents.map(item => item.id);
        const ids = [...previous];
        const from = ids.indexOf(id), to = ids.indexOf(destination);
        ids.splice(from, 1); ids.splice(to, 0, id);
        const saved = await this.perform("Media order saved.", async () => {
            await portfolioApi.reorderMedia(this.projectId, ids, previous);
            await this.refreshMedia();
        });
        if (!saved) this.feedback("Could not save the media order. Use Reload media before trying again.", true);
        const card = this.querySelector(`[data-content-id="${id}"]`);
        const button = card?.querySelector(`[data-move="${direction < 0 ? "up" : "down"}"]`);
        (button && !button.disabled ? button : card?.querySelector('[data-move="handle"]'))?.focus();
    }

    startDrag(event, content) {
        if (this.busy) { event.preventDefault(); return; }
        this.draggedId = content.id;
        event.dataTransfer.effectAllowed = "move";
        event.dataTransfer.setData("text/plain", content.id);
    }

    dragOver(event, content) {
        const source = this.project.contents.find(item => item.id === this.draggedId);
        if (this.busy || !source || source.id === content.id || Boolean(source.behindScenes) !== Boolean(content.behindScenes)) return;
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
        event.currentTarget.classList.add("is-drop-target");
    }

    dropMedia(event, content) {
        event.preventDefault();
        const id = this.draggedId;
        this.endDrag();
        this.moveMedia(id, 0, content.id);
    }

    endDrag() {
        this.draggedId = null;
        this.querySelectorAll(".is-drop-target").forEach(card => card.classList.remove("is-drop-target"));
    }

    async save(event) {
        event.preventDefault();
        const data = new FormData(event.currentTarget);
        const title = data.get("title").trim();
        if (!title) { this.feedback("A title is required.", true); return; }
        await this.perform("Project saved.", async () => {
            await portfolioApi.updateProject(this.projectId, {
                metadata: { title, subTitle: data.get("subTitle").trim(), client: data.get("client").trim() },
                description: { ...this.editor.getContents(), behindScenes: this.scenesEditor.getContents() },
                visible: data.has("visible"),
                adult: data.has("adult"),
            });
            this.querySelector("[data-project-title]").textContent = title;
        });
    }

    async refreshMedia() {
        this.project = await portfolioApi.getProject(this.projectId, { cache: false });
        if (this.isConnected) this.renderMedia();
    }

    async upload(event) {
        event.preventDefault();
        const input = event.currentTarget.querySelector("[name=files]");
        if (!this.uploadQueue.length) this.uploadQueue = [...input.files].map(file => new DirectUpload(portfolioApi, this.projectId, file));
        if (!this.uploadQueue.length) return;
        input.disabled = true;
        await this.perform("Files uploaded.", async () => {
            try {
                while (this.uploadQueue.length) {
                    this.currentUpload = this.uploadQueue[0];
                    this.renderUploadProgress((this.currentUpload.nextPart - 1) * (this.currentUpload.session?.partSize || 0), this.currentUpload.file.size);
                    await this.currentUpload.send((loaded, total) => this.renderUploadProgress(loaded, total));
                    this.uploadQueue.shift();
                    await this.refreshMedia();
                }
                input.value = "";
                render(html``, this.querySelector("[data-upload-progress]"));
            } catch (error) {
                if (!this.currentUpload?.cancelled) this.renderUploadProgress((this.currentUpload.nextPart - 1) * (this.currentUpload.session?.partSize || 0), this.currentUpload?.file.size || 1, true);
                throw error;
            } finally {
                input.disabled = this.uploadQueue.length > 0;
            }
        });
        if (this.currentUpload?.cancelled && this.isConnected) this.feedback("Upload cancelled.");
    }

    renderUploadProgress(loaded, total, failed = false) {
        if (!this.isConnected) return;
        const percent = Math.round(loaded / total * 100);
        render(html`<div class="admin-upload-state"><p>${this.currentUpload.file.name} — ${failed ? "Transfer interrupted" : `${percent}%`}</p>
            <progress max="100" value=${percent} aria-label="Upload progress"></progress>
            <div class="admin-actions">${failed ? html`<button type="button" class="admin-button secondary" @click=${() => this.upload({ preventDefault() {}, currentTarget: this.querySelector(".admin-upload") })}>Retry upload</button>` : ""}
            <button type="button" data-upload-cancel class="admin-button secondary" @click=${() => this.cancelUpload()}>Cancel upload</button></div></div>`, this.querySelector("[data-upload-progress]"));
        this.querySelector("[data-upload-cancel]").disabled = false;
    }

    async cancelUpload() {
        try {
            await this.currentUpload?.abort();
            this.uploadQueue = [];
            this.querySelector("[name=files]").disabled = false;
            this.querySelector("[name=files]").value = "";
            render(html`<p role="status">Upload cancelled.</p>`, this.querySelector("[data-upload-progress]"));
        } catch { this.feedback("Could not cancel the upload. Try again.", true); }
    }

    async saveMedia(event, content) {
        event.preventDefault();
        const data = new FormData(event.currentTarget);
        await this.perform("Media details saved.", async () => {
            await portfolioApi.updateMedia(this.projectId, content.id, {
                caption: data.get("caption").trim(), alt: data.get("alt")?.trim() || "",
                behindScenes: data.get("section") === "scenes", position: Number(data.get("position")), posterId: data.get("posterId") || null,
            });
            await this.refreshMedia();
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
        this.scenesEditor = new Quill(this.querySelector("[data-scenes-editor]"), editorConfig);
        this.scenesEditor.root.setAttribute("aria-label", "Behind the scenes description");
        if (this.project.description?.behindScenes?.ops) this.scenesEditor.setContents(this.project.description.behindScenes);
        this.renderMedia();
    }

    disconnectedCallback() {
        this.editor?.disable();
        this.scenesEditor?.disable();
        this.currentUpload?.abort().catch(() => {});
        super.disconnectedCallback();
    }
}
