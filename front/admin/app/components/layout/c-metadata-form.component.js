import { TemplateRenderer, html } from '../../utils/template-renderer.js';

export class MetadataFormComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
        this.$title = null;
        this.$subTitle = null;
        this.$client = null;
        this.createProject = this.createProject.bind(this);
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
        if (this.$form.checkValidity()) {
            e.preventDefault();
            const title = this.$title.value;
            const subTitle = this.$subTitle.value;
            const client = this.$client.value;
            this.dispatchEvent(new CustomEvent('create-project', { detail: { title, subTitle, client }, bubbles: true, composed: true }))
        }
    }

    cancelCreation = (e) => {
        e.preventDefault();
        this.dispatchEvent(new CustomEvent('cancel-creation', { bubbles: true, composed: true }))
    }

    disconnectedCallback() {
        this.$createProjectButton.removeEventListener('click', e => this.createProject(e));
    }

    connectedCallback() {
        super.connectedCallback();
        this.$createProjectButton = this.querySelector('#createProject');
        this.$title = this.querySelector('#title');
        this.$subTitle = this.querySelector('#subTitle');
        this.$client = this.querySelector('#client');
        this.$form = this.querySelector('.create-project');
        this.$cancelCreationButton = this.querySelector('#cancelCreation');
        this.$createProjectButton.addEventListener('click', e => this.createProject(e));
        this.$cancelCreationButton.addEventListener('click', e => this.cancelCreation(e));
    }

}