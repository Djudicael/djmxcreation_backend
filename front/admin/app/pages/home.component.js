import { TemplateRenderer, html } from "../utils/template-renderer.js";

export class HomeComponent extends TemplateRenderer {
    constructor() { super(); this.noShadow = true; }
    get template() {
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><p class="admin-eyebrow">DJMXCreation / Administration</p><h1>Your creative workspace</h1><p>Prepare your content here, then share it on your portfolio.</p></div></header>
            <div class="admin-dashboard-grid">
                <a class="admin-panel admin-dashboard-link" href="/projects"><span class="admin-badge">01 / Portfolio</span><h2>Your projects</h2><p>Create a project, manage media, and publish your work.</p><span>Manage projects →</span></a>
                <a class="admin-panel admin-dashboard-link" href="/about"><span class="admin-badge">02 / Profile</span><h2>About you</h2><p>Update your name, biography, and profile photo.</p><span>Edit about me →</span></a>
                <a class="admin-panel admin-dashboard-link" href="/contact"><span class="admin-badge">03 / Contact</span><h2>Your contact details</h2><p>Add email addresses, phone numbers, and links so people can contact you.</p><span>Manage contacts →</span></a>
            </div>
        </section>`;
    }
}
