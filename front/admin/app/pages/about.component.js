import { TemplateRenderer, html } from '../utils/template-renderer.js';

import PortfolioApi from '../api/portfolio.api.js';

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
        console.log(this.profilePicture);
        const profilePicture = this.profilePicture ? `<div id=${this.profilePicture.id} class="image-area">
        <img src=${this.profilePicture.url} alt="Preview" loading="lazy">
            <button class="remove-image" data-image-id=${this.profilePicture.id}>delete</button>
        </div>` : '';

        return html`
        <section class="main-content">
            <c-header></c-header>
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
        const blocks = this.description ? this.description : [
            {
                type: 'paragraph',
                data: {
                    text: 'Write description here!'
                }
            }
        ];

        this.$fileCatcher = document.getElementById('file-catcher');
        this.$fileInput = document.getElementById('file-input');
        this.$fileListDisplay = document.getElementById('file-list-display');
        this.$fileInput.addEventListener('change', (event) => {

            for (var i = 0; i < this.$fileInput.files.length; i++) {
                this.fileList.push(this.$fileInput.files[i]);
            }
            this.renderFileList();
        });
        const editor = new EditorJS({
            readOnly: false,
            holder: 'editorjs',
            tools: {
                header: {
                    class: Header,
                    inlineToolbar: ['marker', 'link'],
                    config: {
                        placeholder: 'Header'
                    },
                    shortcut: 'CMD+SHIFT+H'
                },
                image: SimpleImage,

                list: {
                    class: List,
                    inlineToolbar: true,
                    shortcut: 'CMD+SHIFT+L'
                },

                checklist: {
                    class: Checklist,
                    inlineToolbar: true,
                },

                quote: {
                    class: Quote,
                    inlineToolbar: true,
                    config: {
                        quotePlaceholder: 'Enter a quote',
                        captionPlaceholder: 'Quote\'s author',
                    },
                    shortcut: 'CMD+SHIFT+O'
                },

                warning: Warning,

                marker: {
                    class: Marker,
                    shortcut: 'CMD+SHIFT+M'
                },

                code: {
                    class: CodeTool,
                    shortcut: 'CMD+SHIFT+C'
                },

                delimiter: Delimiter,

                inlineCode: {
                    class: InlineCode,
                    shortcut: 'CMD+SHIFT+C'
                },

                linkTool: LinkTool,

                embed: Embed,

                table: {
                    class: Table,
                    inlineToolbar: true,
                    shortcut: 'CMD+ALT+T'
                },

            },
            data: {
                blocks
            },
        });

        const saveButton = document.getElementById('saveButton');

        saveButton.addEventListener('click', async () => {
            const { blocks } = await editor.save().catch((error) => {
                console.error('Saving error', error);
            });

            await this.instance.updateAboutMeDescription(this.id, { description: blocks })
        });
    }

    async getAboutMe() {
        const { id, description, image } = await this.instance.getAboutMe();
        console.log(image);
        this.id = id;
        this.description = description;
        this.profilePicture = image;
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
        const element = e.currentTarget;
        const contentID = element.dataset.imageId;
        this.instance.deleteProfileImage(this.id);
        const card = this.querySelector(`#${contentID}`);
        card.parentNode.removeChild(card);

    };

    initRemoveImageEvent = () => {
        this.querySelectorAll('.remove-image').forEach(item => {
            item.addEventListener('click', e => this.deleteImage(e))
        });
    }

    disconnectedCallback() {
        this.$uploadButton.removeEventListener('click', e => this.sendFile(e));
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