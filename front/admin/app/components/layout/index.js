import { HeaderProjectsManagementComponent } from './c-header-projects.component.js';
import { MetadataFormComponent } from './c-metadata-form.component.js';
import { HeaderComponent } from './c-header-component.js';
import { SideBarComponent } from './c-sidebar-component.js';
import { ShowReelComponent } from './c-showreel-component.js';
import { CardWorkComponent } from './c-card-work-component.js';
import { FooterComponent } from './c-footer-component.js';


export const layoutComponents = [
    {
        tagName: 'c-header',
        component: HeaderComponent
    },
    {
        tagName: 'c-metadata-form',
        component: MetadataFormComponent
    },
    {
        tagName: 'c-header-projects',
        component: HeaderProjectsManagementComponent
    },
    {
        tagName: 'c-sidebar',
        component: SideBarComponent
    },
    {
        tagName: 'c-showreel',
        component: ShowReelComponent
    },
    {
        tagName: 'c-card-work',
        component: CardWorkComponent
    },
    {
        tagName: 'c-footer',
        component: FooterComponent
    },
    // {
    //     tagName: 'c-footer',
    //     component: FooterComponent
    // },
    // {
    //     tagName: 'c-card-lang',
    //     component: LanguageCardComponent
    // }
]