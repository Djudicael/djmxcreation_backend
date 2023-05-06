import { TemplateRenderer, html } from '../utils/template-renderer.js';

import PortfolioApi from '../api/portfolio.api.js';
import Quill from 'quill';

export class AboutComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.instance = new PortfolioApi();
        this.description;
        this.profilePicture;
        this.$fileInput = null;
        this.$fileCatcher = null;
        this.$fileListDisplay = null;
        this.fileList = [];
        this.sendFile = this.sendFile.bind(this);
        this.deleteImage = this.deleteImage.bind(this);
    }

    get template() {
        const profilePicture = this.profilePicture ? html`<div  class="image-area">
        <img src=${this.profilePicture} alt="Preview" loading="lazy">
            <button class="remove-image" data-image-id=${this.profilePicture.id}>delete</button>
        </div>` : '';

        return html`
        <section class="content-page">
            <main class="flex-y">
                <div class="presentation">
                    <div id="editorjs"></div>
                </div>
                <div class="profile">
                    <form id='file-catcher'>
                        <input id='file-input' type='file' multiple />
                        <button class="upload">
                            Submit
                        </button>
                        <section class="flex-x">
                            ${profilePicture}
                        </section>
                    </form>
        
                    <div id='file-list-display'></div>
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
        this.$fileListDisplay.innerHTML = '';
        this.fileList.forEach((file, index) => {
            const fileDisplayEl = document.createElement('p');
            fileDisplayEl.innerHTML = (index + 1) + ': ' + file.name;
            this.$fileListDisplay.appendChild(fileDisplayEl);
        });
    };

    init() {
        this.$fileCatcher = document.getElementById('file-catcher');
        this.$fileInput = document.getElementById('file-input');
        this.$fileListDisplay = document.getElementById('file-list-display');
        this.$fileInput.addEventListener('change', (event) => {

            for (var i = 0; i < this.$fileInput.files.length; i++) {
                this.fileList.push(this.$fileInput.files[i]);
            }
            this.renderFileList();
        });

        const editor = new Quill('#editorjs', {
            theme: 'snow',
            modules: {
                toolbar: [
                    ['bold', 'italic', 'underline', 'strike'],        // toggled buttons
                    ['blockquote', 'code-block'],

                    [{ 'header': 1 }, { 'header': 2 }],               // custom button values
                    [{ 'list': 'ordered' }, { 'list': 'bullet' }],
                    [{ 'script': 'sub' }, { 'script': 'super' }],     // superscript/subscript
                    [{ 'indent': '-1' }, { 'indent': '+1' }],         // outdent/indent
                    [{ 'direction': 'rtl' }],                         // text direction

                    [{ 'size': ['small', false, 'large', 'huge'] }],  // custom dropdown
                    [{ 'header': [1, 2, 3, 4, 5, 6, false] }],

                    [{ 'color': [] }, { 'background': [] }],          // dropdown with defaults from theme
                    [{ 'font': [] }],
                    [{ 'align': [] }],

                    ['clean']                                         // remove formatting button
                ]
            }
        });

        if (this.description) {
            editor.setContents(this.description);
        }

        const saveButton = document.getElementById('saveButton');

        saveButton.addEventListener('click', async () => {
            const blocks = editor.getContents();
            await this.instance.updateAboutMeDescription(this.id, { lastName: this.lastName, firstName: this.firstName, description: blocks })
        });
    }

    async getAboutMe() {
        const { id, firstName, lastName, description, photoUrl } = await this.instance.getAboutMe();
        this.id = id;
        this.firstName = firstName;
        this.lastName = lastName;
        this.description = description;
        this.profilePicture = photoUrl;
        super.render();
    }

    sendFile = async (e) => {
        e.preventDefault();
        for (const file of this.fileList) {
            await this.instance.updateAboutMePicture(this.id, { file });
        }
        await this.getAboutMe();
        this.initRemoveImageEvent();
    };

    deleteImage = async (e) => {
        e.preventDefault();
        console.log("delete");
        this.instance.deleteProfileImage(this.id);

        //TODO change this to a better way
        await this.getAboutMe();
    };

    initRemoveImageEvent = () => {
        this.querySelectorAll('.remove-image').forEach(item => {
            item.addEventListener('click', e => this.deleteImage(e))
        });
    }

    disconnectedCallback() {
        this.$uploadButton.removeEventListener('click', e => this.sendFile(e));
        this.querySelectorAll('.remove-image').forEach(item => {
            item.removeEventListener('click', e => this.deleteImage(e))
        });
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.getAboutMe();
        this.$uploadButton = this.querySelector('.upload');
        this.$uploadButton.addEventListener('click', e => this.sendFile(e));
        this.init();
        this.initRemoveImageEvent();
    }


}