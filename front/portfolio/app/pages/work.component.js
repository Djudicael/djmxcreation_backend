import { TemplateRenderer, html } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
        this.routerOutlet = document.querySelector('router-outlet');
        this.api = new PortfolioApi();
        this.title;
        this.subTitle;
        this.client;
        this.contents;
    }

    get template() {
        const images = this.contents ? html`${this.contents.map(({ url }) =>
            html`
        <c-image cover=${url}></c-image>`)}` : html``;
        return html`
            <div class="content">
                ${images}      
            </div>
        `;
    }

    async getProject(id) {
        const { metadata, description, contents } = await this.api.getProject(id);
        this.title = metadata.title;
        this.subTitle = metadata.subTitle;
        this.client = metadata.client;
        this.description = description;
        this.contents = contents;
        super.render();
    }

    async connectedCallback() {
        super.connectedCallback();
        const location = await this.routerOutlet.getLocation(window.location.pathname);
        const id = location.params.id;
        await this.getProject(id);
    }
}