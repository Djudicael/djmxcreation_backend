import { TemplateRenderer, html } from "../../utils/template-renderer.js";
import { logout } from "../../auth/security.js";

export class SideBarComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.markActive = () => requestAnimationFrame(() => {
            this.querySelectorAll("a").forEach(link => {
                const path = link.getAttribute("href");
                const active = path === "/" ? location.pathname === "/" : location.pathname.startsWith(path);
                if (active) link.setAttribute("aria-current", "page");
                else link.removeAttribute("aria-current");
            });
        });
    }
    get template() {
        return html`<aside class="sidebar admin-sidebar">
            <a class="admin-brand" href="/"><span class="admin-brand-mark">D</span><span>DJMXCreation<small>Administration</small></span></a>
            <nav class="nav" aria-label="Main navigation"><ul>
                <li><a href="/"><span>01</span> Dashboard</a></li>
                <li><a href="/projects"><span>02</span> Projects</a></li>
                <li><a href="/about"><span>03</span> About me</a></li>
                <li><a href="/contact"><span>04</span> Contacts</a></li>
            </ul></nav><div class="admin-sidebar-footer"><button type="button" @click=${event => {
                const button = event.currentTarget;
                button.disabled = true;
                logout().catch(() => { button.disabled = false; button.textContent = "Retry sign out"; });
            }}>Sign out</button><p class="admin-sidebar-note">Your work, your signature.</p></div>
        </aside>`;
    }
    connectedCallback() {
        super.connectedCallback();
        document.addEventListener("click", this.markActive);
        window.addEventListener("popstate", this.markActive);
        this.markActive();
    }
    disconnectedCallback() {
        document.removeEventListener("click", this.markActive);
        window.removeEventListener("popstate", this.markActive);
        super.disconnectedCallback();
    }
}
