import { TemplateRenderer, html } from '../../utils/template-renderer.js';

export class HeaderProjectsManagementComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
        <header class="header">
            <h2>
                <div id="toggle">
                    <img class="header-menu-icon" src="/ressource/icon/menu.svg" />
                </div>
                Dashboard
            </h2>
           
            <div class="user-wrapper">
                <img src="/ressource/profileUser.jpg" width="40px" height="40px" alt="profile" />
                <div>
                    <h4>Judicaël DUBRAY</h4>
                    <small>Super admin</small>
                </div>
        
            </div>
        </header>
        `;
    }

    init() {
        const toggle = document.querySelector('#toggle');

        const sidebar = document.querySelector('.sidebar');
        const mainContent = document.querySelector('.main-content');
        const header = document.querySelector('.header');

        const clickEvent = () => {
            sidebar.classList.toggle('toggled');
            mainContent.classList.toggle('toggled');
            header.classList.toggle('toggled');
        }
        toggle.addEventListener('click', clickEvent);

    }



    connectedCallback() {
        super.connectedCallback();
        this.init();
    }

}