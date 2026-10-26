import { TemplateRenderer, html } from "../../utils/template-renderer";

export default class HeaderComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.isOpen = false;
    }

    get template() {
        return html`
            <a class="wordmark" href="/" aria-label="DJMXCreation home">DJMX<span>Creation</span><span class="brand-dot" aria-hidden="true"></span></a>
            <button class="menu-tog" type="button" aria-controls="portfolio-menu" aria-expanded="false">
                <span class="menu-label">Menu</span><span class="menu-icon" aria-hidden="true"><i></i><i></i></span>
            </button>
            <nav id="portfolio-menu" class="menu" aria-label="Main navigation" aria-hidden="true" inert>
                <div class="links"><ul class="menu-list">
                    ${[["/", "Home", "01"], ["/works", "Works", "02"], ["/about", "About", "03"], ["/contact", "Contact", "04"]].map(([href, label, index]) => html`
                        <li class="menu-wrap"><a href=${href}>${label}/<span aria-hidden="true">${index}</span></a></li>
                    `)}
                </ul></div>
                <span class="menu-signature">DJMXCreation — Portfolio</span>
            </nav>
        `;
    }

    connectedCallback() {
        super.connectedCallback();
        const options = { signal: this.signal };
        this.button = this.querySelector(".menu-tog");
        this.menu = this.querySelector(".menu");
        this.button.addEventListener("click", () => this.setOpen(!this.isOpen), options);
        this.addEventListener("keydown", event => {
            if (!this.isOpen) return;
            if (event.key === "Escape") {
                event.preventDefault();
                this.setOpen(false, true);
            }
            if (event.key === "Tab") {
                const stops = [this.button, ...this.menu.querySelectorAll("a")];
                const first = stops[0], last = stops.at(-1);
                if (event.shiftKey && document.activeElement === first) {
                    event.preventDefault();
                    last.focus();
                } else if (!event.shiftKey && document.activeElement === last) {
                    event.preventDefault();
                    first.focus();
                }
            }
        }, options);
        document.addEventListener("click", event => {
            const anchor = event.target.closest("a[href]");
            if (!anchor || event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
            const url = new URL(anchor.href);
            if (url.origin !== location.origin || anchor.target || anchor.hasAttribute("download") || url.pathname === location.pathname && url.hash) return;
            this.hideMenu();
            requestAnimationFrame(() => {
                window.scrollTo(0, 0);
                this.updateCurrent();
                document.querySelector("main")?.focus({ preventScroll: true });
            });
        }, { ...options, capture: true });
        window.addEventListener("popstate", () => { this.hideMenu(); this.updateCurrent(); }, options);
        this.updateCurrent();
    }

    updateCurrent() {
        const pathname = location.pathname;
        this.menu.querySelectorAll("a").forEach(anchor => {
            const active = anchor.pathname === "/" ? pathname === "/" : pathname.startsWith(anchor.pathname);
            if (active) anchor.setAttribute("aria-current", "page");
            else anchor.removeAttribute("aria-current");
        });
        const section = pathname.split("/")[1];
        document.title = `${({ works: "Works", about: "About", contact: "Contact" })[section] || "Home"} — DJMXCreation`;
    }

    setOpen(open, restoreFocus = false) {
        this.isOpen = open;
        this.menu.inert = !open;
        this.menu.classList.toggle("active", open);
        this.menu.setAttribute("aria-hidden", String(!open));
        this.button.classList.toggle("active", open);
        this.button.setAttribute("aria-expanded", String(open));
        this.button.querySelector(".menu-label").textContent = open ? "Close" : "Menu";
        this.menu.querySelectorAll(".menu-wrap").forEach(item => item.classList.toggle("active", open));
        document.querySelector("main").inert = open;
        this.querySelector(".wordmark").inert = open;
        document.body.classList.toggle("menu-open", open);
        this.updateCurrent();
        if (open) this.menu.querySelector("a[aria-current], a").focus({ preventScroll: true });
        else if (restoreFocus) this.button.focus();
    }

    hideMenu() { if (this.isOpen) this.setOpen(false); }

    disconnectedCallback() {
        this.hideMenu();
        super.disconnectedCallback();
    }
}
