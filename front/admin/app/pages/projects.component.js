import { TemplateRenderer, html, LoadState } from '../utils/template-renderer.js';
import portfolioApi from '../api/portfolio.api.js';
import Metadata from '../models/metadata.js';

export class ProjectManagementComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.instance = portfolioApi;
        this.projects = [];
        this.step = 'PROJECT_HOME';
        this.createProject = this.createProject.bind(this);
        this.deleteProject = this.deleteProject.bind(this);
        this.create = this.create.bind(this);
        this.cancelCreation = this.cancelCreation.bind(this);
        this._onCreateProject = (event) => this.createProject(event);
        this._onDeleteProject = (event) => this.deleteProject(event);
        this._onCancelCreation = (event) => this.cancelCreation(event);
        this.$createButton = null;
    }

    templateEngine({ step }) {
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
        if (this.isLoading && !this.projects.length) return this.loadingTemplate;
        if (this.hasError && !this.projects.length) return this.errorTemplate;

        return html`
        <section class="content-page">
        ${this.creationButton({ step: this.step })}
        <div class="container">
            ${this.templateEngine({ step: this.step })}
        </div>
        </section>`;
    }


    async getProjects() {
        this.setLoadState(LoadState.LOADING);
        this.render();
        try {
            const projects = await this.instance.getProjects();
            if (!this.isConnected) return;
            this.projects = [...projects];
            this.setLoadState(LoadState.DONE);
            super.render();
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load projects.");
            if (this.isConnected) this.render();
        }
    }

    createProject = async (e) => {
        try {
            const metadata = new Metadata({ ...e.detail });
            const { id } = await this.instance.createProject({ ...metadata });
            if (id) {
                const routerOutlet = document.querySelector('router-outlet');
                routerOutlet?.navigateTo?.(`/projects/${id}`);
            }
        } catch (error) {
            console.error('Failed to create project', error);
        }
    }

    cancelCreation = async () => {
        this.step = 'PROJECT_HOME';
        super.render();
        this.initCreationProjectButton();
    }

    initCreationProjectButton() {
        if (this.$createButton) {
            this.$createButton.removeEventListener('click', this.create);
        }
        this.$createButton = this.querySelector('.create-project');
        this.$createButton?.addEventListener('click', this.create);
    }



    create = async (e) => {
        e.preventDefault();
        this.step = 'CREATE_PROJECT';
        super.render();
    }


    deleteProject = async (e) => {
        const { projectId } = e.detail;
        try {
            await this.instance.deleteProject(projectId);
            const element = this.querySelector(`[project-id='${projectId}']`);
            element?.parentNode?.removeChild(element);
            e.preventDefault();
        } catch (error) {
            console.error('Failed to delete project', error);
        }
    };

    disconnectedCallback() {
        this.removeEventListener('create-project', this._onCreateProject);
        this.removeEventListener('delete-project', this._onDeleteProject);
        this.removeEventListener('cancel-creation', this._onCancelCreation);
        this.$createButton?.removeEventListener('click', this.create);
    }


    connectedCallback() {
        super.connectedCallback();
        this.addEventListener('create-project', this._onCreateProject);
        this.addEventListener('delete-project', this._onDeleteProject);
        this.addEventListener('cancel-creation', this._onCancelCreation);
        this.initCreationProjectButton();
        this.getProjects();
    }
}