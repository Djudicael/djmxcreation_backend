import { TemplateRenderer, html } from '../../utils/template-renderer.js';
import { formatDateObject } from '../../lib/date.utils.js';
export class CardWorkComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
        this.deleteProject = this.deleteProject.bind(this);
        this.hideProject = this.hideProject.bind(this);
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
                    <li><button data-project-id=${this.projectId} class="delete-project">
                            <svg version="1.1" id="Layer_1" x="0px" y="0px" width="70px" height="70px" viewBox="0 0 70 70"
                                enable-background="new 0 0 70 70" xml:space="preserve">
                                <g>
                                    <path
                                        d="M58.792,18.241c-0.38-0.442-0.934-0.658-1.517-0.658h-43c-0.583,0-1.137,0.225-1.517,0.667s-0.549,1-0.461,1.576l7,46.02
                                                                                        		c0.149,0.977,0.989,1.737,1.978,1.737h29c0.988,0,1.828-0.75,1.978-1.729l7-46.028C59.341,19.25,59.172,18.684,58.792,18.241z
                                                                                        		 M54.948,21.583l-0.761,5H17.363l-0.761-5H54.948z M48.557,63.583h-4.101l2.643-20.34c0.071-0.549-0.314-1.051-0.862-1.121
                                                                                        		c-0.554-0.076-1.05,0.314-1.12,0.863L42.44,63.583h-5.857v-7.076c0-0.553-0.447-1-1-1s-1,0.447-1,1v7.076h-5.344l-2.973-20.611
                                                                                        		c-0.079-0.547-0.585-0.932-1.133-0.848c-0.547,0.078-0.926,0.586-0.848,1.133l2.932,20.326h-4.223l-5.326-35h4.727l0.884,10.613
                                                                                        		c0.043,0.523,0.48,0.918,0.995,0.918c0.027,0,0.056-0.002,0.084-0.004c0.551-0.047,0.959-0.529,0.913-1.08l-0.871-10.447h10.182
                                                                                        		v23.924c0,0.553,0.447,1,1,1s1-0.447,1-1V28.583h10.399L46.111,39.03c-0.046,0.551,0.362,1.033,0.913,1.08
                                                                                        		c0.028,0.002,0.057,0.004,0.084,0.004c0.515,0,0.952-0.395,0.995-0.918l0.885-10.613h4.895L48.557,63.583z" />
                                    <path
                                        d="M56.725,15.583c1.104,0,2-0.896,2-2s-0.896-2-2-2H43.151c0.075-0.211,0.124-0.435,0.124-0.672c0-4.411-3.589-8-8-8
                                                                                        		s-8,3.589-8,8c0,0.237,0.048,0.461,0.124,0.672H12.725c-1.104,0-2,0.896-2,2s0.896,2,2,2H56.725z M31.275,10.911
                                                                                        		c0-2.206,1.794-4,4-4s4,1.794,4,4c0,0.237,0.049,0.461,0.124,0.672h-8.248C31.227,11.372,31.275,11.148,31.275,10.911z" />
                                </g>
                            </svg>
                        </button></li>
                    <li>
                        <button data-project-id=${this.projectId} class="hide-project ${visibilityClass}">
                            <svg version="1.1" id="Layer_1" x="0px" y="0px" width="70px" height="70px" viewBox="0 0 70 70"
                                enable-background="new 0 0 70 70" xml:space="preserve">
                                <g>
                                    <g>
                                        <path
                                            d="M35,17.5c15.465,0,28,18,28,18s-12.535,17-28,17s-28-17-28-17S19.535,17.5,35,17.5 M35,13.5
                                                                                			c-17.354,0-30.722,18.909-31.282,19.714c-0.979,1.406-0.954,3.28,0.063,4.66C4.342,38.635,17.714,56.5,35,56.5
                                                                                			s30.658-17.865,31.22-18.626c1.017-1.38,1.042-3.254,0.063-4.66C65.722,32.409,52.354,13.5,35,13.5L35,13.5z" />
                                    </g>
                                    <g>
                                        <path
                                            d="M27.912,23.219c-0.394,0-0.768-0.234-0.926-0.622c-0.209-0.511,0.036-1.095,0.548-1.304
                                                                                			c0.797-0.325,1.603-0.604,2.396-0.826c0.533-0.148,1.084,0.16,1.233,0.692s-0.161,1.084-0.692,1.233
                                                                                			c-0.722,0.202-1.455,0.455-2.182,0.752C28.166,23.195,28.038,23.219,27.912,23.219z" />
                                    </g>
                                    <g>
                                        <path
                                            d="M12.188,36.407c-0.197,0-0.396-0.058-0.57-0.18c-0.453-0.315-0.565-0.938-0.249-1.392
                                                                                			c0.167-0.24,4.158-5.921,10.299-10.27c0.452-0.319,1.075-0.212,1.395,0.238c0.319,0.451,0.212,1.075-0.238,1.395
                                                                                			c-5.858,4.147-9.775,9.724-9.814,9.779C12.815,36.258,12.505,36.407,12.188,36.407z" />
                                    </g>
                                    <g>
                                        <path d="M35,27.235c4.564,0,8.265,3.7,8.265,8.265s-3.7,8.265-8.265,8.265s-8.265-3.7-8.265-8.265S30.436,27.235,35,27.235
                                                                                			 M35,25.235c-5.66,0-10.265,4.604-10.265,10.265S29.34,45.765,35,45.765S45.265,41.16,45.265,35.5S40.66,25.235,35,25.235
                                                                                			L35,25.235z" />
                                    </g>
                                    <circle cx="35" cy="35.5" r="3.14" />
                                </g>
                            </svg>
                        </button>
                    </li>
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

    initVisibleButton() {
        this.querySelectorAll('.hide-project').forEach(item => {
            item.addEventListener('click', e => this.hideProject(e))
        });
    }

    initDeleteProjectButton() {
        this.querySelectorAll('.delete-project').forEach(item => {
            item.addEventListener('click', e => this.deleteProject(e))
        });
    }

    deleteProject = async (e) => {
        e.preventDefault();
        const element = e.currentTarget;
        const projectId = element.dataset.projectId
        this.dispatchEvent(new CustomEvent('delete-project', { detail: { projectId }, bubbles: true, composed: true }));

    };

    hideProject = async (e) => {
        e.preventDefault();
        const element = e.currentTarget;
        const projectId = element.dataset.projectId

        console.log("hideProject")

        const visible = !element.classList.contains("visible");
        console.log(visible)
        this.dispatchEvent(new CustomEvent('update-visibility', { detail: { projectId, visible: visible }, bubbles: true, composed: true }));

    };

    attributeChangedCallback(name, oldValue, newValue) {
        if (oldValue !== newValue) {
            console.log('test');
            this.visible = newValue;
            this.render();
            // this.initVisibleButton();
            // this.initDeleteProjectButton();
        }
    }

    connectedCallback() {
        super.connectedCallback();
        this.initVisibleButton();
        this.initDeleteProjectButton();
    }

}