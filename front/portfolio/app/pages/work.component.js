import { TemplateRenderer, html } from "../utils/template-renderer";
import PortfolioApi from "../api/portfolio.api.js";

export default class WorkComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        const menu = document.querySelector('c-header');
        menu.hideMenu();
        this.routerOutlet = document.querySelector('router-outlet');
        this.doc = document.documentElement;
        this.api = new PortfolioApi();
        this.title;
        this.subTitle;
        this.client;
        this.contents;

        this.content;
        // measure translate pixels
        this.current = 0;

        // Store slide number
        this.slide = 0;
        this.init = this.init.bind(this);
        this.appHeight = this.appHeight.bind(this);
        this.startMousedown = this.startMousedown.bind(this);
        this.startTouch = this.startTouch.bind(this);
        this.moveMousedown = this.moveMousedown.bind(this);

    }

    get template() {
        const images = this.contents ? html`${this.contents.map(({ url }) =>
            html`
        <c-image cover=${url}></c-image>`)}` : html``;
        return html`
            <div class="content">
                ${images}      
            </div>
        `;
    }

    async getProject(id) {
        const { metadata, description, contents } = await this.api.getProject(id);
        this.title = metadata.title;
        this.subTitle = metadata.subTitle;
        this.client = metadata.client;
        this.description = description;
        this.contents = contents;
        super.render();
    }

    appHeight = () => {
        this.doc.style.setProperty('--app-height', `${window.innerHeight}px`);
        this.current = -slide * window.innerHeight;
        this.content.style.transform = `translateY(-${slide * window.innerHeight}px)`;

    }

    init() {

        window.addEventListener('resize', this.appHeight)
        this.appHeight();

        mainEl.addEventListener("touchstart", startTouch, { passive: false });
        mainEl.addEventListener("touchend", endTouch, false);
        mainEl.addEventListener("touchmove", moveTouch, { passive: false });
        mainEl.addEventListener("mousedown", startMousedown, false);
        mainEl.addEventListener("mouseup", startMouseup, false);
        mainEl.addEventListener('wheel', wheelFunc, { passive: false })
    }

    wheelFunc(e) {
        if (canSwipe) {
            // swipe up
            if (e.deltaY > 60 && current !== -(window.innerHeight * 5)) {
                canSwipe = false;
                current -= window.innerHeight;
                slide++
                console.log(slide)
                content.style.transform = `translateY(${current}px)`;
                setTimeout(() => {
                    canSwipe = true;
                }, 1000)
            }

            // Swipe down
            if (e.deltaY < -60 && current !== 0) {
                canSwipe = false;
                current += window.innerHeight;
                slide--
                console.log(slide)
                content.style.transform = `translateY(${current}px)`;
                setTimeout(() => {
                    canSwipe = true;
                }, 1000)
            }
        }
    }

    startMousedown(e) {
        initialStart = Date.now();
        initialY = e.clientY;
    }

    startMouseup(e) {
        initialEnd = Date.now();
        endY = e.clientY;
        if (initialEnd - initialStart < 800) {
            swipe()
        }
    }

    async connectedCallback() {
        super.connectedCallback();
        const location = await this.routerOutlet.getLocation(window.location.pathname);
        const id = location.params.id;
        await this.getProject(id);
        this.content = this.shadowRoot.querySelector('.content');
    }
}