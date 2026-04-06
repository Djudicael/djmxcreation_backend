import { TemplateRenderer, html } from '../utils/template-renderer.js';

export class DragDropComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
        this.$fileInput = null;
        this.$fileCatcher = null;
        this.$fileListDisplay = null;
        this.fileList = [];
        this.renderFileList, this.sendFile = null;

        this.uploadImages = this.uploadImages.bind(this);
        this._onUploadClick = this.uploadImages;
        this._onInputChange = null;
        this._onSubmit = null;
    }

    get template() {
        return html`
            <form id='file-catcher'>
                <input id='file-input' type='file' multiple />
                <button class="upload">
                    Submit
                </button>
            </form> 
            
            <div id='file-list-display'></div>
        `;
    }
    init() {
        this.$fileCatcher = this.querySelector('#file-catcher');
        this.$fileInput = this.querySelector('#file-input');
        this.$fileListDisplay = this.querySelector('#file-list-display');

        if (!this.$fileCatcher || !this.$fileInput || !this.$fileListDisplay) {
            return;
        }

        this._onSubmit = (event) => {
            event.preventDefault();
        };
        this.$fileCatcher.addEventListener('submit', this._onSubmit);

        this._onInputChange = () => {

            for (const file of this.$fileInput.files) {
                this.fileList.push(file);
            }
            this.renderFileList();
        };
        this.$fileInput.addEventListener('change', this._onInputChange);

    }
    renderFileList = () => {
        this.$fileListDisplay.innerHTML = '';
        this.fileList.forEach((file, index) => {
            const fileDisplayEl = document.createElement('p');
            fileDisplayEl.innerHTML = (index + 1) + ': ' + file.name;
            this.$fileListDisplay.appendChild(fileDisplayEl);
        });
    };

    uploadImages = (e) => {
        e.preventDefault();

        this.dispatchEvent(new CustomEvent('upload-file', { detail: { files: this.fileList }, bubbles: true, composed: true }));
    }

    disconnectedCallback() {
        this.$uploadButton?.removeEventListener('click', this._onUploadClick);
        this.$fileInput?.removeEventListener('change', this._onInputChange);
        this.$fileCatcher?.removeEventListener('submit', this._onSubmit);
    }

    connectedCallback() {
        super.connectedCallback();
        this.$uploadButton = this.querySelector('.upload');
        this.$uploadButton?.addEventListener('click', this._onUploadClick);
        this.init();
    }

}