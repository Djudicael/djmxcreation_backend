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
    }

    get template() {
        if (this.isLoading) return this.loadingTemplate;
        if (this.hasError) return this.errorTemplate;

        const description = html`${safeHTML(htmlDescription(this.description))}`;

        return html`
        <section class="content-page">
        <div class="container">
        <img src=${this.profileImage} alt="Profile photo" loading="lazy">
        <div class="info">
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
