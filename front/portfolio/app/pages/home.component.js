import { TemplateRenderer, html, LoadState } from "../utils/template-renderer";
import portfolioApi from "../api/portfolio.api.js";

export default class HomeComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        document.querySelector("c-header")?.hideMenu?.();
        this.projects = [];
    }

    get template() {
        const safeProjects = this.projects.filter(project => !project.adult && project.thumbnail?.url);
        const featured = safeProjects[0];
        return html`
            <section class="home-page page-shell" aria-labelledby="home-heading">
                <div class="home-intro">
                    <p class="eyebrow"><span class="small-dot" aria-hidden="true"></span> Portfolio</p>
                    <h1 id="home-heading">DJMX<br><em>Creation</em></h1>
                    <div class="page-actions"><a class="action-link action-primary" href="/works">View works <span aria-hidden="true">↗</span></a><a class="text-link" href="/about">About me <span aria-hidden="true">↗</span></a></div>
                </div>
                <div class="home-feature">
                    ${featured ? html`
                        <a class="featured-image" href=${`/works/${featured.id}`} aria-label=${`View ${featured.metadata?.title || "featured project"}`}>
                            <img src=${featured.thumbnail.url} alt=${featured.metadata?.title || "Featured project"} fetchpriority="high">
                            <span class="image-open" aria-hidden="true">↗</span>
                        </a>
                        <div class="feature-caption"><span class="eyebrow">Featured work</span><a href=${`/works/${featured.id}`}>${featured.metadata?.title} <span aria-hidden="true">↗</span></a></div>
                    ` : html`
                        <a class="visual-study" href="/works" aria-label="Explore the portfolio">
                            <span class="study-orbit orbit-one" aria-hidden="true"></span><span class="study-orbit orbit-two" aria-hidden="true"></span><span class="study-orbit orbit-three" aria-hidden="true"></span>
                            <span class="study-label">Works</span><span class="study-index" aria-hidden="true">01 — ∞</span><span class="image-open" aria-hidden="true">↗</span>
                        </a>
                        <div class="feature-caption"><span class="eyebrow">The portfolio</span><a href="/works">View works <span aria-hidden="true">↗</span></a></div>
                    `}
                </div>
                ${safeProjects.length > 1 ? html`<div class="home-selection"><div class="section-heading"><h2>Selected works</h2><a class="text-link" href="/works">View all works ↗</a></div><div class="selection-grid">
                    ${safeProjects.slice(1, 3).map((project, index) => html`<a class="selection-item" href=${`/works/${project.id}`}><img src=${project.thumbnail.url} alt="" loading="lazy"><span><span class="eyebrow">0${index + 2}</span>${project.metadata?.title}<span aria-hidden="true">↗</span></span></a>`)}
                </div></div>` : ""}
                <div class="home-bottom"><p>DJMXCreation</p><a class="text-link" href="/contact">Contact <span aria-hidden="true">↗</span></a></div>
                ${this.hasError ? html`<p class="quiet-status" role="status">Selected works could not be loaded. <button class="text-link" @click=${() => this.loadProjects()}>Try again</button></p>` : ""}
            </section>
        `;
    }

    async loadProjects() {
        this.setLoadState(LoadState.LOADING);
        try {
            const { projects } = await portfolioApi.getProjects({ page: 1, pageSize: 12 });
            if (!this.isConnected) return;
            this.projects = projects;
            this.setLoadState(LoadState.DONE);
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load projects.");
        }
        if (this.isConnected) this.render();
    }

    connectedCallback() {
        super.connectedCallback();
        this.loadProjects();
    }
}
