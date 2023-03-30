import { TemplateRenderer, html } from '../../utils/template-renderer.js';

export class SideBarComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
        <aside class="sidebar">
            <nav class="nav">
                <ul>
                    <li><a href="/">Home</a></li>
                    <li><a href="/about">About me</a></li>
                    <li><a href="/projects">Projects</a></li>
                    <li><a href="/contact">Contact</a></li>
                </ul>
            </nav>
        </aside>
        `;
    }



    connectedCallback() {
        super.connectedCallback();

    }

}