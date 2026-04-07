import { TemplateRenderer, html } from '../../utils/template-renderer.js';

export class MetadataFormComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
        this.$title = null;
        this.$subTitle = null;
        this.$client = null;
        this.createProject = this.createProject.bind(this);
        this._onCreateProjectClick = (event) => this.createProject(event);
        this._onCancelCreationClick = (event) => this.cancelCreation(event);
    }

    get template() {
        return html`
            <div class="create-project-box">
                <h2>Create portfolio project</h2>
                <form class="create-project">
                    <input id="title" type="text" placeholder="Title" autocomplete="off" required="required" />
                    <input id="subTitle" type="text" placeholder="Subtitle" autocomplete="off" />
                    <input id="client" type="text" placeholder="Netflix" autocomplete="off" />
                    <button id="cancelCreation"> <span>Cancel</span></button>
                    <button id="createProject"> <span>Next</span></button>
                </form>
            </div>       
        `;
    }


    createProject = (e) => {
        e.preventDefault();
        if (!this.$form.checkValidity()) {
            this.$form.reportValidity();
            return;
        }

        const title = this.$title.value.trim();
        const subTitle = this.$subTitle.value.trim();
        const client = this.$client.value.trim();

        if (!title || title.length > 200) {
            this.$title.setCustomValidity('Title must be between 1 and 200 characters');
            this.$form.reportValidity();
            this.$title.setCustomValidity('');
            return;
        }

        this.dispatchEvent(new CustomEvent('create-project', {
            detail: { title, subTitle, client },
            bubbles: true,
            composed: true,
        }));
    }

    cancelCreation = (e) => {
        e.preventDefault();
        this.dispatchEvent(new CustomEvent('cancel-creation', { bubbles: true, composed: true }))
    }

    disconnectedCallback() {
        this.$createProjectButton?.removeEventListener('click', this._onCreateProjectClick);
        this.$cancelCreationButton?.removeEventListener('click', this._onCancelCreationClick);
    }

    connectedCallback() {
        super.connectedCallback();
        this.$createProjectButton = this.querySelector('#createProject');
        this.$title = this.querySelector('#title');
        this.$subTitle = this.querySelector('#subTitle');
        this.$client = this.querySelector('#client');
        this.$form = this.querySelector('.create-project');
        this.$cancelCreationButton = this.querySelector('#cancelCreation');
        this.$createProjectButton?.addEventListener('click', this._onCreateProjectClick);
        this.$cancelCreationButton?.addEventListener('click', this._onCancelCreationClick);
    }

}