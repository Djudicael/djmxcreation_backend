import { TemplateRenderer, html } from '../../utils/template-renderer.js';
export class FooterComponent extends TemplateRenderer {

    constructor() {
        super();
        this.noShadow = true;
    }

    get template() {
        return html`
        <footer class="footer">
            <div class="social">
                <a href="#"><i class="fab fa-linkedin fa-1x"></i></a>
                <a href="#"><i class="fab fa-instagram fa-1x"></i></a>
                <a href="#"><i class="fab fa-twitter fa-1x"></i></a>
            </div>
            <div class="flex">
                <h1>Sylwia Zawiła</h1>
                <p>Copyright &copy; 2021</p>
            </div>
        </footer>
        `;
    }

    connectedCallback() {
        super.connectedCallback();
    }

}