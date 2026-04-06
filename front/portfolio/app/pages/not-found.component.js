import { TemplateRenderer, html } from "../utils/template-renderer.js";

export default class NotFoundComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
      <section class="content-page">
        <div class="container">
          <h2>404</h2>
          <p>Page not found.</p>
        </div>
      </section>
    `;
    }
}
