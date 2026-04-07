import { TemplateRenderer, html } from "../../utils/template-renderer";

export default class HeaderComponent extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.menu = null;
        this.menuTog = null;
        this.menuWraps = null;
        this.wraps = null;
        this.toggleMenu = this.toggleMenu.bind(this);
        this.hideMenu = this.hideMenu.bind(this);
        this._timeouts = [];
    }

    get template() {
        return html`

        <button class="menu-tog" aria-label="Toggle navigation menu" aria-expanded="false">
            <span></span>
            <span></span>
        </button>

        <nav class="menu" role="navigation" aria-label="Main navigation">
            <div class="links">
                <ul class="menu-list" role="menubar">
                    <li class="menu-wrap" role="none"><a href="/" role="menuitem">Home/<span aria-hidden="true">01</span></a></li>
                    <li class="menu-wrap" role="none"><a href="/works" role="menuitem">Works/<span aria-hidden="true">02</span></a></li>
                    <li class="menu-wrap" role="none"><a href="/about" role="menuitem">About/<span aria-hidden="true">03</span></a></li>
                    <li class="menu-wrap" role="none"><a href="/contact" role="menuitem">Contact/<span aria-hidden="true">04</span></a></li>
                </ul>
            </div>
        </nav>

        <div class="header">
            <span class="wrap">
                <h1>Studio</h1>
            </span>
        </div>
        `;
    }

    disconnectedCallback() {
        this.menuTog?.removeEventListener('click', this.toggleMenu);
        this._timeouts.forEach((timerId) => clearTimeout(timerId));
        this._timeouts = [];
    }

    displayWraps() {
        this.wraps.forEach((wrap, idx) => {
            const timerId = setTimeout(() => {
                wrap.classList.add('active');
            }, (idx + 1) * 50);
            this._timeouts.push(timerId);
        })
    }

    hideMenu() {
        if (this.menu?.classList.contains('active')) {
            this.toggleMenu();
        }
    }

    toggleMenu() {
        if (!this.menu || !this.menuTog) {
            return;
        }

        if (this.menu.classList.contains('active')) {
            this.menuTog.classList.remove('active');
            this.menuTog.setAttribute('aria-expanded', 'false');
            this.toggleMenuWraps(false);
            const hideMenuTimer = setTimeout(() => {
                this.menu.classList.remove('active')
            }, 300);
            const showWrapsTimer = setTimeout(() => {
                this.toggleWraps(true);
            }, 300);
            this._timeouts.push(hideMenuTimer, showWrapsTimer);
        } else {
            this.menuTog.classList.add('active');
            this.menuTog.setAttribute('aria-expanded', 'true');
            this.toggleWraps(false);
            const showMenuTimer = setTimeout(() => {
                this.menu.classList.add('active')
            }, 300);
            const showMenuWrapsTimer = setTimeout(() => {
                this.toggleMenuWraps(true);
            }, 300);
            this._timeouts.push(showMenuTimer, showMenuWrapsTimer);
        }
    }

    toggleWraps(active) {
        this.wraps.forEach(wrap => {
            this.toggleWrap(wrap, active);
        })
    }

    toggleMenuWraps(active) {
        this.menuWraps.forEach(wrap => {
            this.toggleWrap(wrap, active);
        })
    }

    toggleWrap(wrap, active) {
        const timerId = setTimeout(() => {
            if (active) {
                wrap.classList.add('active');
            } else {
                wrap.classList.remove('active');
            }
        });
        this._timeouts.push(timerId);
    }

    connectedCallback() {
        super.connectedCallback();
        this.wraps = [...document.querySelectorAll('.wrap')];
        this.menuTog = document.querySelector('.menu-tog');
        this.menu = document.querySelector('.menu');
        this.menuWraps = [...document.querySelectorAll('.menu-wrap')];

        if (!this.menuTog || !this.menu) {
            return;
        }

        this.menuTog.addEventListener('click', this.toggleMenu)

    }
}