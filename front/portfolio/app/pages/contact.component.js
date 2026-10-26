import { TemplateRenderer, html, safeHTML, LoadState } from "../utils/template-renderer";
import portfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";
import { contactHref } from "../../../shared/src/contact-links.js";

export default class ContactComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu?.hideMenu?.();
        this.api = portfolioApi;
        this.description = null;
    }

    get template() {
        if (this.isLoading) return this.loadingTemplate;
        if (this.hasError) return this.errorTemplate;

        const description = html`${safeHTML(htmlDescription(this.description))}`;

        return html`
        <section class="content-page">
        ${description}
        <div class="portfolio-contact-links">${(Array.isArray(this.description?.contacts) ? this.description.contacts : []).map(contact => {
            const href = contactHref(contact);
            return href ? html`<a class="portfolio-contact-link" href=${href}><strong>${contact.label}</strong><span>${contact.value}</span></a>` : "";
        })}</div>
        </section>
        `;
    }

    async connectedCallback() {
        super.connectedCallback();
        this.setLoadState(LoadState.LOADING);
        this.render();
        try {
            const data = await this.api.getContacts();
            if (!this.isConnected) return;
            this.description = data.description;
            this.setLoadState(LoadState.DONE);
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load contact information.");
        }
        this.render();
    }
}
