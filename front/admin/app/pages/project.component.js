import { TemplateRenderer, html } from '../utils/template-renderer.js';
import PortfolioApi from '../api/portfolio.api.js';
import Metadata from '../models/metadata.js';
import ProjectPayload from '../models/projectPayload.js';

export class ProjectComponent extends TemplateRenderer {
    constructor() {
        super();
        this.projectId = null;
        this.noShadow = true;
        this.instance = new PortfolioApi();
        this.title;
        this.subTitle;
        this.client;
        this.visible;
        this.adult;
        this.contents;
        this.deleteImage = this.deleteImage.bind(this);
    }


    get template() {
        const checkVisibility = html`<input type="checkbox" class="toggle" ${this.visible ? 'checked' : ''} >`
        const checkAdult = html`<input type="checkbox" class="adult" ${this.adult ? 'checked' : ''} >`
        const contents = this.contents ? html`${this.contents.map(({ id, url }) => html`
        <div id="area-${id}" class="image-area">
            <img src=${url} alt="Preview">
            <button class="remove-image" data-image-id=${id}>delete</button>
        </div>`)}` : html``;

        return html`
        <section class="content-page">
            <main class="flex-y">
                <div class="form__group field">
                    <div>
                        <label for="title" class="form__label">Title</label>
                        <input type="input" class="form__field" placeholder="Title" name="title" id='title'
                            value=${this.title} />
                    </div>
                    <div>
                        <label for="subtitle" class="form__label">Subtitle</label>
                        <input type="input" class="form__field" placeholder="Subtitle" name="subtitle" id='subtitle'
                            value=${this.subTitle ? this.subTitle : ''} />
                    </div>
                    <div>
                        <label for="client" class="form__label">Client</label>
                        <input type="input" class="form__field" placeholder="Client" name="client" id='client'
                            value=${this.client ? this.client : ''} />
                    </div>
                </div>
                <h1>Project descriptions</h1>
                <div id="editorjs"></div>
                <div class="flex-y">
                    <div>${checkVisibility} <span>Make project
                            visible</span>
                    </div>
                    <div>${checkAdult} <span>This project is for adult</span>
                    </div>
                    <div id="saveButton" class="cta">
                        <span>Save project</span>
                        <svg width="13px" height="10px" viewBox="0 0 13 10">
                            <path d="M1,5 L11,5"></path>
                            <polyline points="8 1 12 5 8 9"></polyline>
                        </svg>
                    </div>
                    <c-drag-drop></c-drag-drop>
                    <section class="flex-x">
                        ${contents}
                    </section>
                </div>
            </main>
        </section>
        `;
    }

    async getProject() {
        const { metadata, visible, description, contents } = await this.instance.getProject(this.projectId);
        this.title = metadata.title;
        this.subTitle = metadata.subTitle;
        this.client = metadata.client;
        this.visible = visible;
        this.description = description;
        this.contents = contents;
        super.render();
    }

    init() {
        const blocks = this.description ? this.description : [
            {
                type: 'paragraph',
                data: {
                    text: 'Write description here!'
                }
            }
        ];
        const editor = new EditorJS({
            /**
             * Enable/Disable the read only mode
             */
            readOnly: false,

            /**
             * Wrapper of Editor
             */
            holder: 'editorjs',

            /**
             * Common Inline Toolbar settings
             * - if true (or not specified), the order from 'tool' property will be used
             * - if an array of tool names, this order will be used
             */
            // inlineToolbar: ['link', 'marker', 'bold', 'italic'],
            // inlineToolbar: true,

            /**
             * Tools list
             */
            tools: {
                /**
                 * Each Tool is a Plugin. Pass them via 'class' option with necessary settings {@link docs/tools.md}
                 */
                header: {
                    class: Header,
                    inlineToolbar: ['marker', 'link'],
                    config: {
                        placeholder: 'Header'
                    },
                    shortcut: 'CMD+SHIFT+H'
                },

                /**
                 * Or pass class directly without any configuration
                 */
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

            /**
             * This Tool will be used as default
             */
            // defaultBlock: 'paragraph',

            /**
             * Initial Editor data
             */
            data: {
                blocks
            },
            // onReady: function () {
            //     saveButton.click();
            // },
            // onChange: function () {
            //     console.log('something changed');
            // }
        });

        /**
         * Saving button
         */
        const saveButton = document.getElementById('saveButton');

        /**
         * Saving contents
         */
        saveButton.addEventListener('click', async () => {
            const { blocks } = await editor.save().catch((error) => {
                console.error('Saving error', error);
            });

            const isVisible = this.querySelector('.toggle').checked;
            const isAdult = this.querySelector('.adult').checked;
            const title = this.querySelector('#title').value;
            const subTitle = this.querySelector('#subtitle').value;
            const client = this.querySelector('#client').value;

            const metadata = new Metadata({ title, subTitle, client });

            const project = new ProjectPayload({ metadata, visible: isVisible, adult: isAdult, description: blocks });
            await this.instance.updateProject(this.projectId, project);
        });
    }

    sendFile = async (e) => {
        for (const file of e.detail.files) {
            await this.instance.addContentToProject(this.projectId, { file });
        }
        await this.getProject();
        this.initRemoveImageEvent();
        //TODO improuve thos system for probably retrieve to push the image after the upload
        super.render();
    };

    deleteImage = async (e) => {
        const element = e.currentTarget;
        const contentID = element.dataset.imageId;
        this.instance.deleteContent(this.projectId, contentID);
        const card = this.querySelector(`#area-${contentID}`);
        // TODO improve the system by refreshing the content and re render the component
        card.parentNode.removeChild(card);
        super.render();
    };

    initRemoveImageEvent = () => {
        this.querySelectorAll('.remove-image').forEach(item => {
            item.addEventListener('click', e => this.deleteImage(e))
        });
    }

    getId = async () => {
        const routerOutlet = document.querySelector('router-outlet');
        const location = await routerOutlet.getLocation(window.location.pathname);
        this.projectId = location.params.id;
    }

    async connectedCallback() {
        super.connectedCallback();
        this.addEventListener('upload-file', e => this.sendFile(e));
        await this.getId();
        await this.getProject();
        this.init();
        this.initRemoveImageEvent();
    }
}