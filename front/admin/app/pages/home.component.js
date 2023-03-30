import { TemplateRenderer, html } from '../utils/template-renderer.js';
import PortfolioApi from '../api/portfolio.api.js';
export class HomeComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.instance = new PortfolioApi();
        this.showReel;
        this.projects = [];
    }

    get template() {


        return html`
        <section class="content-page">
            <div class="container">
                <a target="_blank" href="https://twitter.com/DjmXcreation">
                    <img class="social" src="https://cdn1.iconfinder.com/data/icons/logotypes/32/twitter-128.png">
                </a>
                <p>Follow me</p>
                <p>on Twitter!</p>
            </div>
        </section>
        `;
    }

    async getShowReel() {
        const response = await this.instance.getShowReel();
        this.showReel = response.url
        super.render();
    }

    async getProjects() {
        const response = await this.instance.getProjects();
        this.projects.push(...response);
        super.render();
    }

    connectedCallback() {
        super.connectedCallback();
        // this.getShowReel();
        // this.getProjects();
    }
}