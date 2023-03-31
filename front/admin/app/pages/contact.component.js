import { TemplateRenderer, html } from '../utils/template-renderer.js'

import PortfolioApi from '../api/portfolio.api.js';

export class ContactComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.instance = new PortfolioApi();
        this.id;
        this.description;
    }

    get template() {
        return html`
        <section class="main-content">
            <div id="editorjs"></div>
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
        const blocks = this.description ? this.description : [
            {
                type: 'paragraph',
                data: {
                    text: 'Write contact here!'
                }
            }
        ];

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
            const { blocks } = await editor.save()
                .catch((error) => {
                    console.error('Saving error', error);
                });
            await this.instance.updateContactDescription(this.id, { description: blocks });
        });
    }

    async getContact() {
        const { id, description } = await this.instance.getContacts();
        this.id = id;
        this.description = description;
        super.render();
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.getContact();
        this.init();
    }
}