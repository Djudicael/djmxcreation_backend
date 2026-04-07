import { TemplateRenderer, html } from "../utils/template-renderer.js";

export class HomeComponent extends TemplateRenderer {
  constructor() {
    super();
    this.noShadow = true;
  }

  get template() {
    return html`
      <section class="content-page">
        <div class="container">
          <a target="_blank" rel="noopener noreferrer" href="https://twitter.com/DjmXcreation">X / Twitter</a>
          <p>Follow me</p>
          <p>on Twitter!</p>
        </div>
      </section>
    `;
  }

  connectedCallback() {
    super.connectedCallback();
  }
}
