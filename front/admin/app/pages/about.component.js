import { TemplateRenderer, html, LoadState } from "../utils/template-renderer.js";
import { EventBinder } from "../../../shared/src/event-binder.js";
import Quill from "quill";
import portfolioApi from "../api/portfolio.api.js";
import { editorConfig } from "../utils/helper.js";

export class AboutComponent extends TemplateRenderer {
  constructor() {
    super();
    this.noShadow = true;
    this.instance = portfolioApi;
    this.description = null;
    this.profilePicture = null;
    this.$fileInput = null;
    this.$fileCatcher = null;
    this.$fileListDisplay = null;
    this.fileList = [];
    this.sendFile = this.sendFile.bind(this);
    this.deleteImage = this.deleteImage.bind(this);
    this._removeImageBinder = new EventBinder();
    this._handleFileInputChange = null;
    this._handleSaveClick = null;
    this._handleUploadClick = null;
    this._editor = null;
    this.$saveButton = null;
  }

  get template() {
    if (this.isLoading) return this.loadingTemplate;
    if (this.hasError) return this.errorTemplate;

    const profilePicture = this.profilePicture
      ? html`<div class="image-area">
          <img src=${this.profilePicture} alt="Preview" loading="lazy" />
          <button class="remove-image" data-image-id=${this.profilePicture.id}>
            delete
          </button>
        </div>`
      : "";

    return html`
      <section class="content-page">
        <main class="flex-y">
          <div class="presentation">
            <div id="editorjs"></div>
          </div>
          <div class="profile">
            <form id="file-catcher">
              <input id="file-input" type="file" multiple />
              <button class="upload">Submit</button>
              <section class="flex-x">${profilePicture}</section>
            </form>

            <div id="file-list-display"></div>
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
        </main>
      </section>
    `;
  }

  renderFileList = () => {
    this.$fileListDisplay.innerHTML = "";
    this.fileList.forEach((file, index) => {
      const fileDisplayEl = document.createElement("p");
      fileDisplayEl.textContent = `${index + 1}: ${file.name}`;
      this.$fileListDisplay.appendChild(fileDisplayEl);
    });
  };

  init() {
    this.$fileCatcher = this.querySelector("#file-catcher");
    this.$fileInput = this.querySelector("#file-input");
    this.$fileListDisplay = this.querySelector("#file-list-display");
    this.$saveButton = this.querySelector("#saveButton");

    if (!this.$fileInput || !this.$fileListDisplay || !this.$saveButton) {
      return;
    }

    if (this._handleFileInputChange) {
      this.$fileInput.removeEventListener("change", this._handleFileInputChange);
    }

    this._handleFileInputChange = () => {
      for (const file of this.$fileInput.files) {
        this.fileList.push(file);
      }
      this.renderFileList();
    };
    this.$fileInput.addEventListener("change", this._handleFileInputChange);

    this._editor = new Quill("#editorjs", editorConfig);
    if (this.description) {
      this._editor.setContents(this.description);
    }

    if (this._handleSaveClick) {
      this.$saveButton.removeEventListener("click", this._handleSaveClick);
    }
    this._handleSaveClick = async () => {
      const blocks = this._editor.getContents();
      await this.instance.updateAboutMeDescription(this.id, {
        lastName: this.lastName,
        firstName: this.firstName,
        description: blocks,
      });
    };
    this.$saveButton.addEventListener("click", this._handleSaveClick);
  }

  async getAboutMe() {
    this.setLoadState(LoadState.LOADING);
    this.render();
    try {
      const { id, firstName, lastName, description, photoUrl } =
        await this.instance.getAboutMe();
      if (!this.isConnected) return;
      this.id = id;
      this.firstName = firstName;
      this.lastName = lastName;
      this.description = description;
      this.profilePicture = photoUrl;
      this.setLoadState(LoadState.DONE);
      super.render();
    } catch (error) {
      if (error.name === "AbortError") return;
      this.setLoadState(LoadState.ERROR, "Failed to load about section.");
      if (this.isConnected) this.render();
    }
  }

  sendFile = async (e) => {
    e.preventDefault();
    for (const file of this.fileList) {
      await this.instance.updateAboutMePicture(this.id, { file });
    }
    if (!this.isConnected) {
      return;
    }
    await this.getAboutMe();
    this.initRemoveImageEvent();
  };

  deleteImage = async (e) => {
    e.preventDefault();
    await this.instance.deleteProfileImage(this.id);
    if (!this.isConnected) {
      return;
    }
    await this.getAboutMe();
  };

  initRemoveImageEvent = () => {
    this._removeImageBinder.bindAll(
      this.querySelectorAll(".remove-image"), "click", (e) => this.deleteImage(e)
    );
  };

  disconnectedCallback() {
    if (this.$uploadButton && this._handleUploadClick) {
      this.$uploadButton.removeEventListener("click", this._handleUploadClick);
    }
    if (this.$fileInput && this._handleFileInputChange) {
      this.$fileInput.removeEventListener("change", this._handleFileInputChange);
    }
    if (this.$saveButton && this._handleSaveClick) {
      this.$saveButton.removeEventListener("click", this._handleSaveClick);
    }
    this._removeImageBinder.unbindAll();
  }

  async connectedCallback() {
    super.connectedCallback();
    await this.getAboutMe();
    this.$uploadButton = this.querySelector(".upload");
    this._handleUploadClick = this.sendFile;
    this.$uploadButton?.addEventListener("click", this._handleUploadClick);
    this.init();
    this.initRemoveImageEvent();
  }
}
