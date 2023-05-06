import { TemplateRenderer, html } from '../utils/template-renderer.js'

import PortfolioApi from '../api/portfolio.api.js';
import Quill from 'quill';

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