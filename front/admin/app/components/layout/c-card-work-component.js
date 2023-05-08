import { TemplateRenderer, html } from '../../utils/template-renderer.js';
import { formatDateObject } from '../../lib/date.utils.js';
export class CardWorkComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
        this.deleteProject = this.deleteProject.bind(this);
        this.initDeleteProjectButton = this.initDeleteProjectButton.bind(this);

    }

    static get observedAttributes() {
        return ['visible'];
    }

    get template() {
        const client = this.client ? `<span class="client">Client ${this.client}</span>` : '';
        const visibilityClass = this.visible == 'true' ? 'visible' : '';
        return html`
    <div class="project card">
        <div class="wrapper" style="background: url(${this.cover}) center/cover no-repeat;">
            <div class="header">
                <div class="date">
                    <span class="day">${this.creationDate.day}</span>
                    <span>${this.creationDate.date}</span>
                    <span class="month">${this.creationDate.monthName}</span>
                    <span class="year">${this.creationDate.year}</span>
                </div>
                <ul class="menu-content">
                    <button data-project-id=${this.projectId} class="delete-project">
                            Delete project
                        </button>
                </ul>
            </div>
            <div class="data">
                <div class="content">
                    ${client}
                    <h1 class="title"><a href="#">${this.title}</a></h1>
                    <p class="text">${this.subTitle}</p>
                    <a href="/projects/${this.projectId}" data-navigo class="button">Modify</a>
                </div>
            </div>
        </div>
        `;
    }

    get projectId() {
        return this.getAttribute('project-id');
    }

    set projectId(val) {
        if (val) {
            this.setAttribute('project-id', val);
        } else {
            this.removeAttribute('project-id');
        }
    }
    get title() {
        return this.getAttribute('title');
    }

    set title(val) {
        if (val) {
            this.setAttribute('title', val);
        } else {
            this.removeAttribute('title');
        }
    }

    get subTitle() {
        return this.getAttribute('subtitle');
    }

    set subTitle(val) {
        if (val) {
            this.setAttribute('subtitle', val);
        } else {
            this.removeAttribute('subtitle');
        }
    }

    get client() {
        return this.getAttribute('client');
    }

    set client(val) {
        if (val) {
            this.setAttribute('client', val);
        } else {
            this.removeAttribute('client');
        }
    }
    get creationDate() {
        return formatDateObject(this.getAttribute('creation-date'));
    }

    set creationDate(val) {
        if (val) {
            this.setAttribute('creation-date', val);
        } else {
            this.removeAttribute('creation-date');
        }
    }
    get visible() {
        return this.getAttribute('visible');
    }

    set visible(val) {
        if (val) {
            this.setAttribute('visible', val);
        } else {
            this.removeAttribute('visible');
        }
    }

    get cover() {
        return this.getAttribute('cover');
    }

    set cover(val) {
        if (val) {
            this.setAttribute('cover', val);
        } else {
            this.removeAttribute('cover');
        }
    }



    initDeleteProjectButton() {

        const deleteProjectButton = this.querySelector('.delete-project');
        console.log(deleteProjectButton);
        deleteProjectButton.addEventListener('click', this.deleteProject)

        // this.querySelectorAll('.delete-project').forEach(item => {
        //     console.log(item);
        // });
    }

    deleteProject = async (e) => {
        e.preventDefault();
        console.log('delete project');
        const element = e.currentTarget;
        const projectId = element.dataset.projectId
        this.dispatchEvent(new CustomEvent('delete-project', { detail: { projectId }, bubbles: true, composed: true }));

    };



    attributeChangedCallback(name, oldValue, newValue) {
        if (oldValue !== newValue) {
            this.visible = newValue;
            this.render();
        }
    }

    connectedCallback() {
        super.connectedCallback();
        this.render();
        this.initDeleteProjectButton();
    }

}