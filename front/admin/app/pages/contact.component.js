import Quill from "quill";
import PortfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";
import { TemplateRenderer, html } from "../utils/template-renderer.js";

export class ContactComponent extends TemplateRenderer {
  constructor() {
    super();
    this.noShadow = true;
    this.instance = new PortfolioApi();
    this.id;
    this.description;
    this._editor = null;
    this.$saveButton = null;
    this._onSaveClick = null;
  }

  get template() {
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
    try {
      const { id, description } = await this.instance.getContacts();
      this.id = id;
      this.description = description;
      if (this.isConnected) {
        super.render();
      }
    } catch (error) {
      console.error("Failed to load contact", error);
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
