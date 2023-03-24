import { TemplateRenderer, html } from '../../utils/template-renderer.js';

export class SideBarComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
        <div class="sidebar">
            <div class="side-brand">
                <h2>Judicaël DUBRAY</h2>
            </div>
            <div class="sidebar-menu">
                <ul>
                    <li>
                        <a href="$1" class="active"><img src="/ressource/icon/diagram.svg" /><span>Dashboard</span></a>
                    </li>
                    <li>
                        <a href="$1"><img src="/ressource/icon/boy.svg" /><span>Profile</span></a>
                    </li>
                    <li>
                        <a href="$1"><img src="/ressource/icon/group.svg" /><span>Users</span></a>
                    </li>
                    <li>
                        <a href="/projects"><img src="/ressource/icon/folder.svg" /><span>Projects</span></a>
                    </li>
                    <li>
                        <a href="/about"><img src="/ressource/icon/about.svg" data-navigo /><span>About me</span></a>
                    </li>
                    <li>
                        <a href="/contact"><img src="/ressource/icon/pen tool.svg" data-navigo /><span>Contacts</span></a>
                    </li>
                </ul>
            </div>
        </div>
        `;
    }



    connectedCallback() {
        super.connectedCallback();

    }

}