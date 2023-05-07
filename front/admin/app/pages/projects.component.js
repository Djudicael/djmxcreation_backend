import { TemplateRenderer, html, sanitizeHtml } from '../utils/template-renderer.js';
import PortfolioApi from '../api/portfolio.api.js';
import Metadata from '../models/metadata.js';

export class ProjectManagementComponent extends TemplateRenderer {
    // TODO https://codepen.io/choogoor/pen/YWBxAg
    constructor(router) {
        super();
        this.noShadow = true;
        this.router = router;
        this.instance = new PortfolioApi();
        this.projects = [];
        this.step = 'PROJECT_HOME';
        this.createProject = this.createProject.bind(this);
        this.deleteProject = this.deleteProject.bind(this);
        this.create = this.create.bind(this);
        this.cancelCreation = this.cancelCreation.bind(this);
    }

    templateEngine({ step }) {
        console.log(this.projects);
        switch (step) {
            case 'CREATE_PROJECT':
                return html`<c-metadata-form></c-metadata-form>`
            case 'PROJECT_HOME':
            default:
                return this.projects
                    ? html`${this.projects.map(
                        ({ id, createdOn, metadata, visible, thumbnail }) =>
                            html`<c-card-work
                        id=${id}
                        project-id=${id}
                        title=${metadata.title}
                        ${metadata.sub_title ? `subtitle=${metadata.sub_title}` : ''}
                        ${metadata.client ? `client=${metadata.client}` : ''}
                        creation-date=${createdOn}
                        visible=${visible}
                        cover=${(thumbnail && thumbnail.url) ? thumbnail.url : '/ressource/icon/boy.svg'}
                        ></c-card-work>`
                    )}`
                    : '';
        }
    }


    creationButton({ step }) {
        switch (step) {
            case 'PROJECT_HOME':
                return html`<button class="create-project">New project</button>`;
            default:
                return html``;
        }
    }


    get template() {
        return html`
        <section class="content-page">
        ${this.creationButton({ step: this.step })}
        <div class="container">
            ${this.templateEngine({ step: this.step })}
        </div>
        </section>`;
    }


    async getProjects() {
        const projects = await this.instance.getProjects();
        this.projects.push(...projects);
        super.render();
    }

    createProject = async (e) => {
        const metadata = new Metadata({ ...e.detail });
        const { id } = await this.instance.createProject({ ...metadata });
        if (id) {
            const routerOutlet = document.querySelector('router-outlet');
            routerOutlet.navigateTo(`/projects/${id}`);
        }
    }

    cancelCreation = async (e) => {
        this.step = 'PROJECT_HOME';
        super.render();
        this.initCreationProjectButton();
    }

    initCreationProjectButton() {
        this.$createButton = document.querySelector('.create-project');
        this.$createButton.addEventListener('click', this.create);
    }



    create = async (e) => {
        e.preventDefault();
        this.step = 'CREATE_PROJECT';
        super.render();
    }


    deleteProject = async (e) => {
        const { projectId } = e.detail;
        await this.instance.deleteProject(projectId);
        const element = this.querySelector(`[project-id='${projectId}']`);
        element.parentNode.removeChild(element);
        e.preventDefault();
    };

    disconnectedCallback() {
        this.removeEventListener('create-project', e => this.createProject(e));
        this.removeEventListener('delete-project', e => this.deleteProject(e));
        this.removeEventListener('cancel-creation', e => this.cancelCreation(e));
    }


    connectedCallback() {
        super.connectedCallback();
        this.addEventListener('create-project', e => this.createProject(e));
        this.addEventListener('delete-project', e => this.deleteProject(e));
        this.addEventListener('cancel-creation', e => this.cancelCreation(e));
        // this.remove
        this.initCreationProjectButton();
        this.getProjects();
    }
}