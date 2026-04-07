import Quill from "quill";
import portfolioApi from "../api/portfolio.api.js";
import Metadata from "../models/metadata.js";
import ProjectPayload from "../models/projectPayload.js";
import { editorConfig } from "../utils/helper.js";
import { TemplateRenderer, html, LoadState } from "../utils/template-renderer.js";
import { EventBinder } from "../../../shared/src/event-binder.js";

export class ProjectComponent extends TemplateRenderer {
  constructor() {
    super();
    this.projectId = null;
    this.noShadow = true;
    this.instance = portfolioApi;
    this.title = null;
    this.subTitle = null;
    this.client = null;
    this.visible = false;
    this.adult = false;
    this.contents = null;
    this.thumbnail = null;
    this.deleteImage = this.deleteImage.bind(this);
    this.thumbImage = this.thumbImage.bind(this);
    this._onUploadFile = (event) => this.sendFile(event);
    this._onSaveClick = null;
    this._removeImageBinder = new EventBinder();
    this._thumbImageBinder = new EventBinder();
    this._editor = null;
  }

  getFragmentWithVisibility(visible) {
    if (visible) {
      return html`<input type="checkbox" class="toggle" checked />`;
    }
    return html`<input type="checkbox" class="toggle" />`;
  }
  getFragmentWithAdult(adult) {
    if (adult) {
      return html`<input type="checkbox" class="adult" checked />`;
    }
    return html`<input type="checkbox" class="adult" />`;
  }

  get template() {
    if (this.isLoading) return this.loadingTemplate;
    if (this.hasError) return this.errorTemplate;

    const checkVisibility = this.getFragmentWithVisibility(this.visible);
    const checkAdult = this.getFragmentWithAdult(this.adult);

    const contents = this.contents
      ? html`${this.contents.map(
        ({ id, url }) => html`
            <div id="area-${id}" class="image-area">
              <img src=${url} alt="Project content" loading="lazy" />
              <button class="thumb-image" data-image-id=${id} aria-label="Set as thumbnail">thumb</button>
              <button class="remove-image" data-image-id=${id} aria-label="Delete image">delete</button>
            </div>
          `
      )}`
      : html``;

    const thumbnail = this.thumbnail
      ? html`
          <div id="thumbnail-area" class="image-area">
            <img src=${this.thumbnail} alt="Project thumbnail" loading="lazy" />
          </div>
        `
      : html``;

    return html`
      <section class="content-page">
        <main class="flex-y">
          <div class="form__group field">
            <div>
              <label for="title" class="form__label">Title</label>
              <input
                type="input"
                class="form__field"
                placeholder="Title"
                name="title"
                id="title"
                value=${this.title}
              />
            </div>
            <div>
              <label for="subtitle" class="form__label">Subtitle</label>
              <input
                type="input"
                class="form__field"
                placeholder="Subtitle"
                name="subtitle"
                id="subtitle"
                value=${this.subTitle ? this.subTitle : ""}
              />
            </div>
            <div>
              <label for="client" class="form__label">Client</label>
              <input
                type="input"
                class="form__field"
                placeholder="Client"
                name="client"
                id="client"
                value=${this.client ? this.client : ""}
              />
            </div>
          </div>

          <h2>Thumbnail</h2>
          ${thumbnail}

          <h1>Project descriptions</h1>
          <div id="editorjs"></div>
          <div class="flex-y">
            <div>${checkVisibility} <span>Make project visible</span></div>
            <div>${checkAdult} <span>This project is for adult</span></div>
            <div id="saveButton" class="cta">
              <span>Save project</span>
              <svg width="13px" height="10px" viewBox="0 0 13 10">
                <path d="M1,5 L11,5"></path>
                <polyline points="8 1 12 5 8 9"></polyline>
              </svg>
            </div>
            <c-drag-drop></c-drag-drop>
            <section class="flex-x">${contents}</section>
          </div>
        </main>
      </section>
    `;
  }

  async getProject() {
    this.setLoadState(LoadState.LOADING);
    this.render();
    try {
      const { metadata, visible, description, contents, thumbnail, adult } =
        await this.instance.getProject(this.projectId);
      if (!this.isConnected) return;
      this.title = metadata.title;
      this.subTitle = metadata.subTitle;
      this.client = metadata.client;
      this.visible = visible;
      this.adult = adult;
      this.description = description;
      this.contents = contents;

      if (thumbnail) {
        this.thumbnail = thumbnail.url;
      }

      this.setLoadState(LoadState.DONE);
      super.render();
    } catch (error) {
      if (error.name === "AbortError") return;
      this.setLoadState(LoadState.ERROR, "Failed to load project.");
      if (this.isConnected) this.render();
    }
  }

  init() {
    const saveButton = this.querySelector("#saveButton");
    if (!saveButton) {
      return;
    }

    this._editor = new Quill("#editorjs", editorConfig);

    if (this.description) {
      this._editor.setContents(this.description);
    }

    if (this._onSaveClick) {
      saveButton.removeEventListener("click", this._onSaveClick);
    }

    this._onSaveClick = async () => {
      const blocks = this._editor.getContents();

      const isVisible = this.querySelector(".toggle").checked;
      const isAdult = this.querySelector(".adult").checked;
      const title = this.querySelector("#title").value;
      const subTitle = this.querySelector("#subtitle").value;
      const client = this.querySelector("#client").value;

      const metadata = new Metadata({ title, subTitle, client });

      const project = new ProjectPayload({
        metadata,
        visible: isVisible,
        adult: isAdult,
        description: blocks,
      });
      await this.instance.updateProject(this.projectId, project);
    };
    saveButton.addEventListener("click", this._onSaveClick);
  }

  sendFile = async (e) => {
    for (const file of e.detail.files) {
      await this.instance.addContentToProject(this.projectId, { file });
    }
    if (!this.isConnected) {
      return;
    }
    await this.getProject();
    this.initRemoveImageEvent();
    this.initThumbImageEvent();
  };

  deleteImage = async (e) => {
    const element = e.currentTarget;
    const contentID = element.dataset.imageId;
    await this.instance.deleteContent(this.projectId, contentID);
    if (!this.isConnected) {
      return;
    }
    const card = this.querySelector(`#area-${contentID}`);
    card?.parentNode?.removeChild(card);
    this.thumbnail = null;
    super.render();
    this.initRemoveImageEvent();
    this.initThumbImageEvent();
  };

  thumbImage = async (e) => {
    const element = e.currentTarget;
    const contentID = element.dataset.imageId;
    const thumbnail = await this.instance.addThumbnail(
      this.projectId,
      contentID
    );
    if (!this.isConnected) {
      return;
    }
    const url = thumbnail.url;
    this.thumbnail = url;
    super.render();
    this.initRemoveImageEvent();
    this.initThumbImageEvent();
  };

  initRemoveImageEvent = () => {
    this._removeImageBinder.bindAll(
      this.querySelectorAll(".remove-image"), "click", (e) => this.deleteImage(e)
    );
  };

  initThumbImageEvent = () => {
    this._thumbImageBinder.bindAll(
      this.querySelectorAll(".thumb-image"), "click", (e) => this.thumbImage(e)
    );
  };

  getId = async () => {
    const routerOutlet = document.querySelector("router-outlet");
    const location = await routerOutlet.getLocation(window.location.pathname);
    this.projectId = location.params.id;
  };

  async connectedCallback() {
    super.connectedCallback();
    this.addEventListener("upload-file", this._onUploadFile);
    await this.getId();
    await this.getProject();
    this.init();
    this.initRemoveImageEvent();
    this.initThumbImageEvent();
  }

  disconnectedCallback() {
    this.removeEventListener("upload-file", this._onUploadFile);
    this._removeImageBinder.unbindAll();
    this._thumbImageBinder.unbindAll();

    const saveButton = this.querySelector("#saveButton");
    if (saveButton && this._onSaveClick) {
      saveButton.removeEventListener("click", this._onSaveClick);
    }
  }
}
