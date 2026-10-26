import Quill from "quill";
import { render } from "lit-html";
import { html } from "../utils/template-renderer.js";
import { AdminPage } from "../utils/admin-page.js";
import portfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";
import { contactHref } from "../../../shared/src/contact-links.js";

export class ContactComponent extends AdminPage {
    constructor() { super(); this.contact = null; this.contacts = []; }

    get template() {
        if (!this.contact) return html`<section class="content-page admin-page"><p role="status">Loading contacts…</p></section>`;
        return html`<section class="content-page admin-page">
            <header class="admin-page-header"><div><p class="admin-eyebrow">Get in touch</p><h1>Contacts</h1><p>Add the contact details and links to display on your portfolio.</p></div></header>
            ${this.statusTemplate}
            <form class="admin-panel" @submit=${event => this.save(event)}>
                <h2>Contact details and links</h2><div data-contacts></div>
                <button class="admin-button secondary" type="button" @click=${() => this.addContact()}>+ Add contact</button>
                <h2 class="admin-section-title">Contact page text</h2><p>Describe your availability or the kinds of collaborations you are looking for.</p>
                <div data-editor></div>
                <div class="admin-actions"><button class="admin-button" type="submit">Save contacts</button></div>
            </form>
        </section>`;
    }

    renderContacts() {
        render(this.contacts.length ? html`<div class="admin-contact-list">${this.contacts.map((contact, index) => html`
            <fieldset class="admin-contact-row"><legend>Contact ${index + 1}</legend>
                <label>Label<input required maxlength="120" .value=${contact.label} @input=${event => { contact.label = event.target.value; }} /></label>
                <label>Type<select .value=${contact.type} @change=${event => { contact.type = event.target.value; this.renderContacts(); }}>
                    <option value="email">Email</option><option value="phone">Phone</option><option value="url">Website or social profile</option>
                </select></label>
                <label>Contact detail<input required type=${contact.type === "email" ? "email" : contact.type === "phone" ? "tel" : "url"} maxlength="500"
                    .value=${contact.value} placeholder=${contact.type === "email" ? "hello@example.com" : contact.type === "phone" ? "+33 6 12 34 56 78" : "https://…"}
                    @input=${event => { contact.value = event.target.value; }} /></label>
                <button class="admin-button danger" type="button" aria-label=${`Remove contact ${index + 1}`} @click=${() => {
                    this.contacts.splice(index, 1); this.renderContacts(); this.feedback("Save your changes to apply this removal.");
                }}>Remove</button>
            </fieldset>`)}</div>` : html`<p class="admin-empty-inline">No contact details yet. Add your first contact below.</p>`, this.querySelector("[data-contacts]"));
    }

    addContact() {
        this.contacts.push({ id: crypto.randomUUID(), type: "email", label: "Email", value: "" });
        this.renderContacts();
        const rows = this.querySelectorAll(".admin-contact-row");
        rows[rows.length - 1]?.querySelector("input")?.focus();
    }

    async save(event) {
        event.preventDefault();
        const contacts = this.contacts.map(contact => ({ ...contact, label: contact.label.trim(), value: contact.value.trim() }));
        if (contacts.some(contact => !contactHref(contact))) {
            this.feedback("Check your contact details: use a valid email, phone number, or HTTP(S) link.", true);
            return;
        }
        await this.perform("Contacts saved.", () => portfolioApi.updateContactDescription(this.contact.id, {
            description: { ...this.contact.description, ...this.editor.getContents(), contacts },
        }));
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.load(async () => {
            this.contact = await portfolioApi.getContacts();
            this.contacts = Array.isArray(this.contact.description?.contacts) ? structuredClone(this.contact.description.contacts) : [];
        });
        if (!this.isConnected || !this.contact) return;
        this.editor = new Quill(this.querySelector("[data-editor]"), editorConfig);
        this.editor.root.setAttribute("aria-label", "Contact page text");
        if (this.contact.description?.ops) this.editor.setContents(this.contact.description);
        this.renderContacts();
    }

    disconnectedCallback() {
        this.editor?.disable();
        super.disconnectedCallback();
    }
}
