import { TemplateRenderer, html, safeHTML, LoadState } from "../utils/template-renderer";
import portfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";

export default class AboutComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu?.hideMenu?.();
        this.api = portfolioApi;
        this.description = null;
        this.profileImage = null;
        this.displayName = "";
    }

    get template() {
        if (this.isLoading) return this.loadingTemplate;
        if (this.hasError) return this.errorTemplate;

        const description = html`${safeHTML(htmlDescription(this.description))}`;

        return html`
        <section class="about-page page-shell">
            <p class="eyebrow">03 / About</p>
            <div class="about-layout ${this.profileImage ? "" : "without-portrait"}">
                ${this.profileImage ? html`<figure class="about-portrait"><img src=${this.profileImage} alt=${this.displayName ? `Portrait of ${this.displayName}` : "Profile portrait"} loading="lazy"></figure>` : ""}
                <div class="about-copy"><h1>${this.displayName || "About me."}</h1><div class="rich-text">${description}</div>
                    <div class="page-actions"><a class="action-link action-primary" href="/works">View works <span aria-hidden="true">↗</span></a><a class="text-link" href="/contact">Contact ↗</a></div>
                </div>
            </div>
        </section>
        `;
    }

    async connectedCallback() {
        super.connectedCallback();
        this.setLoadState(LoadState.LOADING);
        this.render();
        try {
            const data = await this.api.getAboutMe();
            this.displayName = [data.firstName, data.lastName].filter(Boolean).join(" ");
            if (!this.isConnected) return;
            this.description = data.description;
            this.profileImage = data.photoUrl;
            this.setLoadState(LoadState.DONE);
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load about section.");
        }
        this.render();
    }
}
