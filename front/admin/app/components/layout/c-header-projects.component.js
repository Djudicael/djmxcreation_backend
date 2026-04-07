import { HeaderComponent } from './c-header-component.js';
import { html } from '../../utils/template-renderer.js';

export class HeaderProjectsManagementComponent extends HeaderComponent {

    get template() {
        return html`
        <header class="header">
            <h2>
                <div id="toggle">
                    <img class="header-menu-icon" src="/ressource/icon/menu.svg" alt="Toggle menu" />
                </div>
                Dashboard
            </h2>

            <div class="user-wrapper">
                <img src="/ressource/profileUser.jpg" width="40px" height="40px" alt="profile" />
                <div>
                    <h4>Judicael DUBRAY</h4>
                    <small>Super admin</small>
                </div>

            </div>
        </header>
        `;
    }
}
