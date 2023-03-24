import { TemplateRenderer } from '../utils/template-renderer.js';
export class FeatureComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return `
        <section id="featured">
            <div class="title">
                Featured
            </div>
            <div class="videos-section">
                <div class="video"></div>
                <div class="video"></div>
                <div class="video center"></div>
                <div class="video"></div>
                <div class="video"></div>
                <div class="arrows">
                    <div class="arrow"><i class="fas fa-arrow-left"></i></div>
                    <div class="arrow"><i class="fas fa-arrow-right"></i></div>
                </div>

                <div class="circles">
                    <div class="circle"></div>
                    <div class="circle"></div>
                    <div class="circle active"></div>
                    <div class="circle"></div>
                    <div class="circle"></div>
                </div>
            </div>
        </section>
        `;
    }

    connectedCallback() {
        super.connectedCallback();
    }

}