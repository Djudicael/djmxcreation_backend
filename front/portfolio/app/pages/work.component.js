import { TemplateRenderer, html } from "../utils/template-renderer";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
    }

    get template() {
        return html`
        <section class="content-page">
            <p>This is work page</p>
        </section>
        `;
    }
}