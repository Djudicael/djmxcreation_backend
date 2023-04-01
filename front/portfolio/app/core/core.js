import { ComponentRegistry } from './component-registry.js';
import { layoutComponents } from '../components/layout/index.js';
//PAGES
import HomeComponent from '../pages/home.component.js';
import WorkComponent from '../pages/work.component.js';
import WorksComponent from '../pages/works.component.js';
import AboutComponent from '../pages/about.component.js';
import ContactComponent from '../pages/contact.component.js';
import RouterOutlet from '../controller/router-outlet.js';


export class Core {
    constructor() {
        if (!Core.inst) {
            Core.inst = this;
        } else {
            throw new Error('use instance');
        }
        ComponentRegistry.register(components);

        return Core.inst;
    }
    static get instance() {
        return Core.inst;
    }
}

Core.inst = null;

const components = [
    ...layoutComponents,
    {
        tagName: 'p-home',
        component: HomeComponent
    },
    {
        tagName: 'p-work',
        component: WorkComponent
    },
    {
        tagName: 'p-about',
        component: AboutComponent
    },
    {
        tagName: 'p-contact',
        component: ContactComponent
    },
    {
        tagName: 'p-works',
        component: WorksComponent
    },
    {
        tagName: 'router-outlet',
        component: RouterOutlet
    },
];