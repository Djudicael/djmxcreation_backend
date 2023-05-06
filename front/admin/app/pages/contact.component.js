import { TemplateRenderer, html } from '../utils/template-renderer.js'

import PortfolioApi from '../api/portfolio.api.js';
import Quill from 'quill';
import { editorConfig } from '../utils/helper.js';

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
        const editor = new Quill('#editorjs', editorConfig);

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