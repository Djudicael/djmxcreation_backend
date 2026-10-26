import { TemplateRenderer, html } from "../utils/template-renderer.js";

export default class NotFoundComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
      <section class="page-shell">
        <p class="eyebrow">404 / A wrong turn</p>
        <h1>Page not found.</h1>
        <div class="page-actions">
          <a class="action-link action-primary" href="/">Back to home ↗</a>
          <a class="text-link" href="/works">Explore the works</a>
        </div>
      </section>
    `;
    }
}
