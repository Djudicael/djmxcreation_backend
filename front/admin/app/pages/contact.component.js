import Quill from "quill";
import portfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";
import { TemplateRenderer, html, LoadState } from "../utils/template-renderer.js";

export class ContactComponent extends TemplateRenderer {
  constructor() {
    super();
    this.noShadow = true;
    this.instance = portfolioApi;
    this.id = null;
    this.description = null;
    this._editor = null;
    this.$saveButton = null;
    this._onSaveClick = null;
  }

  get template() {
    if (this.isLoading) return this.loadingTemplate;
    if (this.hasError) return this.errorTemplate;

    return html`
      <section class="content-page">
        <div class="presentation">
          <div id="editorjs"></div>
        </div>
        <div class="flex-y">
          <div id="saveButton" class="cta">
            <span>Save project</span>
            <svg width="13px" height="10px" viewBox="0 0 13 10">
              <path d="M1,5 L11,5"></path>
              <polyline points="8 1 12 5 8 9"></polyline>
            </svg>
          </div>
        </div>
      </section>
    `;
  }

  init() {
    this._editor = new Quill("#editorjs", editorConfig);

    if (this.description) {
      this._editor.setContents(this.description);
    }

    this.$saveButton = this.querySelector("#saveButton");
    if (!this.$saveButton) {
      return;
    }

    if (this._onSaveClick) {
      this.$saveButton.removeEventListener("click", this._onSaveClick);
    }

    this._onSaveClick = async () => {
      const blocks = this._editor.getContents();
      await this.instance.updateContactDescription(this.id, {
        description: blocks,
      });
    };

    this.$saveButton.addEventListener("click", this._onSaveClick);
  }

  async getContact() {
    this.setLoadState(LoadState.LOADING);
    this.render();
    try {
      const { id, description } = await this.instance.getContacts();
      if (!this.isConnected) return;
      this.id = id;
      this.description = description;
      this.setLoadState(LoadState.DONE);
      super.render();
    } catch (error) {
      if (error.name === "AbortError") return;
      this.setLoadState(LoadState.ERROR, "Failed to load contact information.");
      if (this.isConnected) this.render();
    }
  }

  async connectedCallback() {
    super.connectedCallback();
    await this.getContact();
    if (!this.isConnected) {
      return;
    }
    this.init();
  }

  disconnectedCallback() {
    if (this.$saveButton && this._onSaveClick) {
      this.$saveButton.removeEventListener("click", this._onSaveClick);
    }
  }
}
