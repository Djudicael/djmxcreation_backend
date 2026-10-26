import Quill from "quill";
import { render } from "lit-html";
import { html } from "../utils/template-renderer.js";
import { AdminPage } from "../utils/admin-page.js";
import portfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";

export class AboutComponent extends AdminPage {
    constructor() { super(); this.about = null; }

    get template() {
        if (!this.about) return html`<section class="content-page admin-page"><p role="status">Loading about me…</p></section>`;
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><p class="admin-eyebrow">Your profile</p><h1>About me</h1><p>This information appears on the About page of your portfolio.</p></div></header>
            ${this.statusTemplate}
            <form class="admin-panel" @submit=${event => this.save(event)}>
                <h2>Identity and biography</h2><div class="admin-fields">
                    <label>First name<input name="firstName" required maxlength="200" .value=${this.about.firstName || ""} /></label>
                    <label>Last name<input name="lastName" required maxlength="200" .value=${this.about.lastName || ""} /></label>
                </div>
                <label class="admin-editor-label">About me</label><div data-editor></div>
                <div class="admin-actions"><button class="admin-button" type="submit">Save about me</button></div>
            </form>
            <section class="admin-panel"><h2>Profile photo</h2><div data-photo></div>
                <form class="admin-upload" @submit=${event => this.upload(event)}>
                    <label>New photo<input name="photo" type="file" accept="image/*" required /></label>
                    <button class="admin-button secondary" type="submit">Upload photo</button>
                </form>
            </section>
        </section>`;
    }

    renderPhoto() {
        render(this.about.photoUrl ? html`<div class="admin-profile-photo"><img src=${this.about.photoUrl} alt="Current profile photo" />
            <button class="admin-button danger" @click=${() => this.removePhoto()}>Delete photo</button></div>` : html`<p class="admin-muted">No profile photo.</p>`, this.querySelector("[data-photo]"));
    }

    async save(event) {
        event.preventDefault();
        const data = new FormData(event.currentTarget);
        await this.perform("About me saved.", () => portfolioApi.updateAboutMeDescription(this.about.id, {
            firstName: data.get("firstName").trim(), lastName: data.get("lastName").trim(), description: this.editor.getContents(),
        }));
    }

    async refreshPhoto() {
        this.about = await portfolioApi.getAboutMe();
        if (this.isConnected) this.renderPhoto();
    }

    async upload(event) {
        event.preventDefault();
        const input = event.currentTarget.querySelector("[name=photo]");
        const file = input.files[0];
        if (!file) return;
        await this.perform("Photo uploaded.", async () => {
            if (file.size > 32 * 1024 * 1024) throw new Error("The photo exceeds the 32 MiB limit.");
            await portfolioApi.updateAboutMePicture(this.about.id, { file });
            await this.refreshPhoto();
            input.value = "";
        });
    }

    async removePhoto() {
        if (!window.confirm("Delete the profile photo?")) return;
        await this.perform("Photo deleted.", async () => {
            await portfolioApi.deleteProfileImage(this.about.id);
            await this.refreshPhoto();
        });
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.load(async () => { this.about = await portfolioApi.getAboutMe(); });
        if (!this.isConnected || !this.about) return;
        this.editor = new Quill(this.querySelector("[data-editor]"), editorConfig);
        this.editor.root.setAttribute("aria-label", "About me");
        if (this.about.description?.ops) this.editor.setContents(this.about.description);
        this.renderPhoto();
    }

    disconnectedCallback() {
        this.editor?.disable();
        super.disconnectedCallback();
    }
}
