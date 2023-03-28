import { TemplateRenderer } from '../utils/template-renderer.js';
import { Router } from '@vaadin/router';
export class RouterOutlet extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }


    getTemplate() {
        return `
             <div id="router-outlet">Router outlet ....</div>
        `;
    }

    connectedCallback() {
        super.connectedCallback();
        const router = new Router(document.querySelector('router-outlet'));

        router.setRoutes([
            { path: '/', component: 'p-home' },
            { path: '/about', component: 'p-about' },
            { path: '/contact', component: 'p-contact' },
            { path: '/work', component: 'p-work' },
            { path: '/services', component: 'p-services' },
            { path: '/projects', component: 'p-project-management' },
            { path: '/project', component: 'p-project' },
        ]);
    }
}