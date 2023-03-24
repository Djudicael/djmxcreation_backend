import { TemplateRenderer, html, sanitizeHtml } from '../utils/template-renderer.js';
import PortfolioApi from '../api/portfolio.api.js';

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
        this.hideProject = this.hideProject.bind(this);
        this.create = this.create.bind(this);
    }

    templateEngine({ step }) {
        console.log(this.projects);
        switch (step) {
            case 'CREATE_PROJECT':
                return `<c-metadata-form></c-metadata-form>`
            case 'PROJECT_HOME':
            default:
                return this.projects ? this.projects.map(({ id, created_on, metadata, visible, content }) => `<c-card-work id=${id} project-id=${id} title=${metadata.title} ${metadata.subTitle ? `subtitle=${metadata.subTitle}` : ''}  ${metadata.client ? `client=${metadata.client}` : ''} creation-date=${created_on} visible=${visible} cover=${(content && content.length) ? content[0].url : '/ressource/icon/boy.svg'}></c-card-work>`).join('') : '';
        }
    }


    creationButton({ step }) {
        switch (step) {
            case 'PROJECT_HOME':
                return ` <button class="create-project">New project</button>`;
            default:
                return ``;
        }
    }


    get template() {

       
        return html`<section class="main-content">
        <c-header></c-header>
        <main>
            ${this.creationButton({ step: this.step })}
    
            ${this.templateEngine({ step: this.step })}
        </main>
    </section>
        `;
    }


    async getProjects() {
        const projects = await this.instance.getProjects();
        this.projects.push(...projects);
        super.render();
        this.router.updatePageLinks();
    }

    createProject = async (e) => {
        const { id } = await this.instance.createProject({ ...e.detail });
        if (id) {
            this.router.navigate(`/projects/${id}`);
        }
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
        this.instance.deleteProject(projectId);
        const element = this.querySelector(`[project-id='${projectId}']`);
        element.parentNode.removeChild(element);
        e.preventDefault();
    };

    hideProject = async (e) => {
        const { projectId, visible } = e.detail;
        e.preventDefault();
        this.instance.updateVisibility(projectId, { isVisible: visible });
        const element = this.querySelector(`[project-id='${projectId}']`);
        element.setAttribute("visible", visible.toString());

    };


    connectedCallback() {
        super.connectedCallback();
        this.addEventListener('create-project', e => this.createProject(e));
        this.addEventListener('delete-project', e => this.deleteProject(e));
        this.addEventListener('update-visibility', e => this.hideProject(e));
        this.initCreationProjectButton();
        this.getProjects();
    }
}