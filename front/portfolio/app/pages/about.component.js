import { TemplateRenderer, html, unsafeHTML } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";

export default class AboutComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
        this.api = new PortfolioApi();
        this.description;
        this.profileImage;
    }

    get template() {
        const description = html`${unsafeHTML(htmlDescription(this.description))}`;

        return html`
        <section class="content-page">
        <div class="container">
		<img src=${this.profileImage} alt="Me">
		<div class="info">
        ${description}
        </div>
	</div>
        </section>
        `;
    }

    async connectedCallback() {
        super.connectedCallback();
        const data = await this.api.getAboutMe();
        this.description = data.description;
        this.profileImage = data.photoUrl;
        this.render();
    }
}