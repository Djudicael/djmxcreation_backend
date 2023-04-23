import { TemplateRenderer, html } from "../utils/template-renderer";

import PortfolioApi from "../api/portfolio.api";

function lerp(start, end, t) {
    return start * (1 - t) + end * t;
}


export default class WorksComponent extends TemplateRenderer {
    constructor() {
        super();
        const menu = document.querySelector('c-header');
        menu.hideMenu()
        this.routerOutlet = document.querySelector('router-outlet');
        this.noShadow = true;
        this.api = new PortfolioApi();
        this.projects = [];
        this.canvas = null;
        this.ctx = null;
        this.links = [];
        this.page = 1;
        this.pageSize = 100;
        this.totalPages = 0;

        // Canvas mousemove variables

        this.targetX = 0;
        this.targetY = 0;
        this.currentX = 0;
        this.currentY = 0;

        this.percent = 0;
        this.target = 0;
        this.imgArr = [];
        this.imgIndex = 0;
        this.init = this.init.bind(this);
        this.getThumbnailImage = this.getThumbnailImage.bind(this);
        this.animate = this.animate.bind(this);
    }

    get template() {
        const projects = this.projects ? html`${this.projects.map(({ id, metadata }) => html`<li class="project" project-id=${id}>${metadata.title}</li>`)}` : html``;
        return html`
        <canvas></canvas>

        <section class="projects-section">
            <div class="projects">
                <ul>
                    ${projects}
                </ul>
            </div>
        </section>
        `;
    }

    async getProjects() {
        const { totalPages, page, size, projects } = await this.api.getProjects({ page: this.page, pageSize: this.pageSize });
        this.totalPages = totalPages;
        this.page = page;
        this.pageSize = size;
        console.log(projects);
        this.projects.push(...projects);
        super.render();
    }

    drawImage(idx) {
        let { width, height } = this.imgArr.filter(img => img.projectId == idx)[0].elImage.getBoundingClientRect();

        this.canvas.width = width * window.devicePixelRatio;
        this.canvas.height = height * window.devicePixelRatio;
        this.canvas.style.width = `${width}px`;
        this.canvas.style.height = `${height}px`;

        // pixelate by diabling the smoothing
        this.ctx.webkitImageSmoothingEnabled = false;
        this.ctx.mozImageSmoothingEnabled = false;
        this.ctx.msSmoothingEnabled = false;
        this.ctx.imageSmoothingEnabled = false;

        if (this.target === 1) { // Link has been hovered
            // 2 speeds to make the effect more gradual
            if (this.percent < 0.2) {
                this.percent += .01;
            } else if (this.percent < 1) {
                this.percent += .1;
            }
        } else if (this.target === 0) {
            if (this.percent > 0.2) {
                this.percent -= .3
            } else if (this.percent > 0) {
                this.percent -= .01;
            }
        }

        let scaledWidth = width * this.percent;
        let scaledHeight = height * this.percent;

        if (this.percent >= 1) {
            this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
            this.ctx.drawImage(this.imgArr.filter(img => img.projectId == idx)[0].elImage, 0, 0, width, height);
        } else {
            this.ctx.drawImage(this.imgArr.filter(img => img.projectId == idx)[0].elImage, 0, 0, scaledWidth, scaledHeight);
            this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
            if (this.canvas.width !== 0 && this.canvas.height !== 0) {
                this.ctx.drawImage(this.canvas, 0, 0, scaledWidth, scaledHeight, 0, 0, width, height)
            }
        }
    }

    init() {


        this.imgIndex = this.projects[0].id;

        this.canvas = document.querySelector('canvas');
        this.ctx = this.canvas.getContext('2d');

        this.links = [...document.querySelectorAll('.project')];


        for (const link of this.links) {

            const projectId = link.getAttribute('project-id');
            let image = this.getThumbnailImage(projectId);
            let elImage = new Image(300);
            elImage.src = image;
            elImage.classList.add('project-image');
            document.body.append(elImage);
            this.imgArr.push({ projectId, elImage })
        }


        window.addEventListener('mousemove', (e) => {
            this.targetX = e.clientX;
            this.targetY = e.clientY;

        });


        for (const link of this.links) {
            const projectId = link.getAttribute('project-id');

            link.addEventListener('mouseover', () => {

                for (const linkInternal of this.links) {
                    const projectInternalId = linkInternal.getAttribute('project-id');
                    if (projectInternalId !== projectId) {
                        linkInternal.style.opacity = 0.2;
                        linkInternal.style.zIndex = 0;
                    } else {
                        linkInternal.style.opacity = 1;
                        linkInternal.style.zIndex = 3;
                    }
                }
            })

            link.addEventListener('mouseleave', () => {
                for (const linkMouseLeave of this.links) {
                    linkMouseLeave.style.opacity = 1;
                }
            })

            link.addEventListener('mouseenter', () => {
                this.imgIndex = projectId;
                this.target = 1
            });

            link.addEventListener('mouseleave', () => {
                this.target = 0;
            });

            link.addEventListener('click', () => {
                this.routerOutlet.navigateTo(`/works/${projectId}`);
            });

        }

    }

    getThumbnailImage(imgIndex) {
        const projectImage = this.projects.filter(project => project.id == imgIndex).map(project => project.thumbnail.url)[0];
        return projectImage;
    }

    animate() {
        this.currentX = lerp(this.currentX, this.targetX, 0.075);
        this.currentY = lerp(this.currentY, this.targetY, 0.075);
        const image = this.imgArr.filter(img => img.projectId == this.imgIndex)[0].elImage;
        let { width, height } = image.getBoundingClientRect();
        this.canvas.style.transform = `translate3d(${this.currentX - (width / 2)}px, ${this.currentY - (height / 2)}px, 0)`;
        this.drawImage(this.imgIndex);
        window.requestAnimationFrame(this.animate);
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.getProjects();

        if (this.projects.length) {
            this.init();
            this.animate();
        }
    }
}