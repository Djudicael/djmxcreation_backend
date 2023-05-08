import { TemplateRenderer, html, unsafeHTML } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";
import { htmlDescription } from "../utils/helper.js";


export default class ContactComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
        this.api = new PortfolioApi();
        this.description;
    }


    get template() {
        const description = html`${unsafeHTML(htmlDescription(this.description))}`;

        return html`
        <section class="content-page">
        ${description}
        </section>
        `;
    }

    async connectedCallback() {
        super.connectedCallback();
        const data = await this.api.getContacts();
        this.description = data.description;
        this.render();
    }
}