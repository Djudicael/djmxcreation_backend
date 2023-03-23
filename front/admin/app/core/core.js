import { ComponentRegistry } from './component-registry.js';
import { layoutComponents } from '../components/layout/index.js';
//PAGES
import { HomeComponent } from '../pages/home.component.js';
import { DragDropComponent } from '../components/c-drag-drop.component.js';
import { ProjectManagementComponent } from '../pages/projects.component.js';
import { LoginComponent } from '../pages/login.component.js';
import { ServiceComponent } from '../pages/service.component.js';
import { WorkComponent } from '../pages/work.component.js';
import { AboutComponent } from '../pages/about.component.js';
import { ContactComponent } from '../pages/contact.component.js';
import { ProjectComponent } from '../pages/project.component.js';


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
        tagName: 'c-login',
        component: LoginComponent
    },
    {
        tagName: 'c-drag-drop',
        component: DragDropComponent
    },
    {
        tagName: 'c-project-management',
        component: ProjectManagementComponent
    },
    {
        tagName: 'c-home',
        component: HomeComponent
    },
    {
        tagName: 'c-service',
        component: ServiceComponent
    },
    {
        tagName: 'c-work',
        component: WorkComponent
    },
    {
        tagName: 'c-about',
        component: AboutComponent
    },
    {
        tagName: 'c-contact',
        component: ContactComponent
    },
    {
        tagName: 'c-project',
        component: ProjectComponent
    },
];