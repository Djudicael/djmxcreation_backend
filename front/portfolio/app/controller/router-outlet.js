import { TemplateRenderer } from '../utils/template-renderer.js';
import { Router } from '../../../shared/src/router.js';
export default class RouterOutlet extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
        this.navigateTo = this.navigateTo.bind(this);
        this.getLocation = this.getLocation.bind(this);
    }


    getTemplate() {
        return `
            <div id="router-outlet">Router outlet ....</div>
        `;
    }

    navigateTo(path) {
        Router.go(path);
    }

    async getLocation(pathname) {
        if (!this.router) {
            return null;
        }
        const location = await this.router.resolve(pathname);
        return location;
    }


    connectedCallback() {
        super.connectedCallback();
        const router = new Router(this);

        router.setRoutes([
            { path: '/', component: 'p-home' },
            { path: '/works', component: 'p-works' },
            { path: '/works/:id', component: 'p-work' },
            { path: '/about', component: 'p-about' },
            { path: '/contact', component: 'p-contact' },
            { path: '*', component: 'p-not-found' },
        ]);
        this.router = router;

    }

    disconnectedCallback() {
        this.router?.dispose?.();
    }
}