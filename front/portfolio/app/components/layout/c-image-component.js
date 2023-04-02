import { TemplateRenderer, html } from "../../utils/template-renderer";

export default class ImageComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
        <section class="section">
            <div class="img active" style="background: url(${this.cover})"></div>
        </section>
        `;
    }

    get cover() {
        return this.getAttribute('cover');
    }

    set cover(val) {
        if (val) {
            this.setAttribute('cover', val);
        } else {
            this.removeAttribute('cover');
        }
    }

    connectedCallback() {
        super.render();
    }
}