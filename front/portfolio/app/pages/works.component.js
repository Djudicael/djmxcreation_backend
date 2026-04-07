import { TemplateRenderer, html, safeHTML, LoadState } from "../utils/template-renderer";

import portfolioApi from "../api/portfolio.api.js";
import { lerp } from "../utils/helper.js";
export default class WorksComponent extends TemplateRenderer {
    constructor() {
        super();
        const menu = document.querySelector('c-header');
        menu?.hideMenu?.();
        this.routerOutlet = document.querySelector('router-outlet');
        this.noShadow = true;
        this.api = portfolioApi;
        this.projects = [];
        this.canvas = null;
        this.ctx = null;
        this.links = [];
        this.page = 1;
        this.pageSize = 6;
        this.totalPages = 0;

        // Canvas mousemove variables

        this.targetX = 0;
        this.targetY = 0;
        this.currentX = 0;
        this.currentY = 0;

        this.percent = 0;
        this.target = 0;
        this.imgArr = [];
        this._imgMap = new Map();
        this.imgIndex = 0;
        this._mouseMoveHandler = null;
        this._linkHandlers = [];
        this._onLoadMoreClick = null;
        this._frameId = null;
        this._loadMoreButton = null;
        this.init = this.init.bind(this);
        this.getThumbnailImage = this.getThumbnailImage.bind(this);
        this.animate = this.animate.bind(this);
    }

    nsfwFragment(adult) {
        if (adult) {
            return '<span class="nsfw">|NSFW🔞</span>';
        }
        return '';
    }

    get template() {
        if (this.isLoading && !this.projects.length) return this.loadingTemplate;
        if (this.hasError && !this.projects.length) return this.errorTemplate;

        const projects = this.projects ? html`${this.projects.map(({ id, metadata, adult }) => html`<li class="project" role="button" tabindex="0" project-id=${id}>${metadata.title} ${safeHTML(this.nsfwFragment(adult))} </li>`)}` : html``;
        const loadMore = this.page < this.totalPages ? html`<div class="load-more-container">
        <button class="load-more" aria-label="Load more projects">
            <span class="button-text">Load More</span>
            <span class="button-arrow"></span>
        </button>
    </div>` : html``;
        return html`
        <canvas></canvas>

        <section class="projects-section">
            <div class="projects">
                <ul>
                    ${projects}
                </ul>
               ${loadMore}
            </div>
        </section>
        `;
    }

    async getProjects() {
        this.setLoadState(LoadState.LOADING);
        try {
            const { totalPages, page, size, projects } = await this.api.getProjects({ page: this.page, pageSize: this.pageSize });
            if (!this.isConnected) return;
            this.totalPages = totalPages;
            this.page = page;
            this.pageSize = size;
            this.projects.push(...projects);
            this.setLoadState(LoadState.DONE);
            super.render();
        } catch (error) {
            if (error.name === "AbortError") return;
            this.setLoadState(LoadState.ERROR, "Failed to load works.");
            if (this.isConnected) super.render();
        }
    }

    drawImage(idx) {
        const imageEntry = this._imgMap.get(String(idx));
        if (!imageEntry || !this.canvas || !this.ctx) {
            return;
        }

        let { width, height } = imageEntry.elImage.getBoundingClientRect();

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
            this.ctx.drawImage(imageEntry.elImage, 0, 0, width, height);
        } else {
            this.ctx.drawImage(imageEntry.elImage, 0, 0, scaledWidth, scaledHeight);
            this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
            if (this.canvas.width !== 0 && this.canvas.height !== 0) {
                this.ctx.drawImage(this.canvas, 0, 0, scaledWidth, scaledHeight, 0, 0, width, height)
            }
        }
    }

    cleanupDynamicResources() {
        if (this._mouseMoveHandler) {
            window.removeEventListener('mousemove', this._mouseMoveHandler);
            this._mouseMoveHandler = null;
        }

        this._linkHandlers.forEach(({ link, handlers }) => {
            link.removeEventListener('mouseover', handlers.onMouseOver);
            link.removeEventListener('mouseleave', handlers.onMouseLeaveOpacity);
            link.removeEventListener('mouseenter', handlers.onMouseEnter);
            link.removeEventListener('mouseleave', handlers.onMouseLeaveTarget);
            link.removeEventListener('click', handlers.onClick);
            link.removeEventListener('keydown', handlers.onKeydown);
        });
        this._linkHandlers = [];

        if (this._loadMoreButton && this._onLoadMoreClick) {
            this._loadMoreButton.removeEventListener('click', this._onLoadMoreClick);
        }
        this._loadMoreButton = null;
        this._onLoadMoreClick = null;

        if (this._frameId) {
            window.cancelAnimationFrame(this._frameId);
            this._frameId = null;
        }

        this.imgArr.forEach(({ elImage }) => elImage.remove());
        this.imgArr = [];
        this._imgMap.clear();
    }

    init() {
        if (!this.projects.length) {
            return;
        }

        this.cleanupDynamicResources();

        this.imgIndex = this.projects[0].id;

        this.canvas = this.querySelector('canvas');
        if (!this.canvas) {
            return;
        }
        this.ctx = this.canvas.getContext('2d');

        this.links = [...this.querySelectorAll('.project')];


        for (const link of this.links) {

            const projectId = link.getAttribute('project-id');
            let image = this.getThumbnailImage(projectId);
            let elImage = new Image(300);
            elImage.src = image;
            elImage.classList.add('project-image');
            document.body.append(elImage);
            const entry = { projectId: String(projectId), elImage };
            this.imgArr.push(entry)
            this._imgMap.set(String(projectId), entry);
        }

        this._mouseMoveHandler = (e) => {
            this.targetX = e.clientX;
            this.targetY = e.clientY;
        };
        window.addEventListener('mousemove', this._mouseMoveHandler);


        for (const link of this.links) {
            const projectId = String(link.getAttribute('project-id'));

            const onMouseOver = () => {

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
            };

            const onMouseLeaveOpacity = () => {
                for (const linkMouseLeave of this.links) {
                    linkMouseLeave.style.opacity = 1;
                }
            };

            const onMouseEnter = () => {
                this.imgIndex = projectId;
                this.target = 1
            };

            const onMouseLeaveTarget = () => {
                this.target = 0;
            };

            const onClick = () => {
                this.routerOutlet?.navigateTo?.(`/works/${projectId}`);
            };

            const onKeydown = (e) => {
                if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    onClick();
                }
            };

            link.addEventListener('mouseover', onMouseOver);
            link.addEventListener('mouseleave', onMouseLeaveOpacity);
            link.addEventListener('mouseenter', onMouseEnter);
            link.addEventListener('mouseleave', onMouseLeaveTarget);
            link.addEventListener('click', onClick);
            link.addEventListener('keydown', onKeydown);

            this._linkHandlers.push({
                link,
                handlers: {
                    onMouseOver,
                    onMouseLeaveOpacity,
                    onMouseEnter,
                    onMouseLeaveTarget,
                    onClick,
                    onKeydown,
                },
            });

        }

    }

    getThumbnailImage(imgIndex) {
        const project = this.projects.find((item) => item.id == imgIndex);
        return project?.thumbnail?.url || '/ressource/icon/boy.svg';
    }

    animate() {
        if (!this.isConnected || !this.imgArr.length || !this.canvas) {
            return;
        }

        this.currentX = lerp(this.currentX, this.targetX, 0.075);
        this.currentY = lerp(this.currentY, this.targetY, 0.075);
        const imageEntry = this._imgMap.get(String(this.imgIndex));
        if (!imageEntry) {
            return;
        }
        const image = imageEntry.elImage;
        let { width, height } = image.getBoundingClientRect();
        this.canvas.style.transform = `translate3d(${this.currentX - (width / 2)}px, ${this.currentY - (height / 2)}px, 0)`;
        this.drawImage(this.imgIndex);
        this._frameId = window.requestAnimationFrame(this.animate);
    }

    nextPage() {
        if (this._loadMoreButton && this._onLoadMoreClick) {
            this._loadMoreButton.removeEventListener('click', this._onLoadMoreClick);
        }

        this._loadMoreButton = this.querySelector('.load-more');
        if (!this._loadMoreButton) {
            return;
        }

        this._onLoadMoreClick = async () => {
            if (this.page < this.totalPages) {
                this.page++;
                await this.getProjects();
                if (!this.isConnected) {
                    return;
                }
                this.init();
                this.animate();
            }
        };

        this._loadMoreButton.addEventListener('click', this._onLoadMoreClick);
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.getProjects();
        if (!this.isConnected) {
            return;
        }
        this.nextPage();

        if (this.projects.length) {
            this.init();
            this.animate();
        }
    }

    disconnectedCallback() {
        super.disconnectedCallback();
        this.cleanupDynamicResources();
    }
}