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
        <section class="content-page">
        <div class="container">
        ${this.profileImage ? html`<img src=${this.profileImage} alt="Profile photo" loading="lazy">` : ""}
        <div class="info">
        <h1>${this.displayName}</h1>
        ${description}
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
