import { TemplateRenderer, html } from "../utils/template-renderer";

export default class AboutComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
    }

    get template() {
        return html`
        <section class="content-page">
            <p>This is about page</p>
        </section>
        `;
    }
}