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
        this.$fileCatcher = document.getElementById('file-catcher');
        this.$fileInput = document.getElementById('file-input');
        this.$fileListDisplay = document.getElementById('file-list-display');
        this.$fileCatcher.addEventListener('submit', function (evnt) {
            evnt.preventDefault();
            this.fileList.forEach(function (file) {
                sendFile(file);
            });
        });

        this.$fileInput.addEventListener('change', (event) => {

            for (const file of this.$fileInput.files) {
                this.fileList.push(file);
            }
            this.renderFileList();
        });

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
        this.$uploadButton.removeEventListener('click', e => this.uploadImages(e));
    }

    connectedCallback() {
        super.connectedCallback();
        this.$uploadButton = this.querySelector('.upload');
        this.$uploadButton.addEventListener('click', e => this.uploadImages(e));
        this.init();
    }

}