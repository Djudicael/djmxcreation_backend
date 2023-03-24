import { TemplateRenderer, html } from '../../utils/template-renderer.js';

export class ShowReelComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
        <section class="showreel">
        <iframe src=${this.showreel} width="640" height="360" frameborder="0"
            allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>
        </section>
        `;
    }

    get showreel() {
        return this.getAttribute('showreel');
    }

    set showreel(val) {
        if (val) {
            this.setAttribute('showreel', val);
        } else {
            this.removeAttribute('showreel');
        }
    }

    connectedCallback() {
        super.connectedCallback();
    }

}