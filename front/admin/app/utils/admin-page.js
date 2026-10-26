import { TemplateRenderer, html } from "./template-renderer.js";

export class AdminPage extends TemplateRenderer {
    constructor() {
        super();
        this.noShadow = true;
        this.busy = false;
    }

    feedback(message, error = false) {
        const status = this.querySelector("[data-status]");
        if (!status) return;
        status.textContent = message;
        status.classList.toggle("is-error", error);
        status.setAttribute("role", error ? "alert" : "status");
    }

    async perform(message, operation) {
        if (this.busy) return;
        this.busy = true;
        this.setAttribute("aria-busy", "true");
        this.querySelectorAll("button").forEach(button => { button.disabled = true; });
        this.feedback("Working…");
        try {
            await operation();
            if (this.isConnected) this.feedback(message);
            return true;
        } catch (error) {
            if (this.isConnected) this.feedback(`The operation failed. ${error.message}`, true);
            return false;
        } finally {
            this.busy = false;
            this.removeAttribute("aria-busy");
            this.querySelectorAll("button").forEach(button => { button.disabled = false; });
        }
    }

    get statusTemplate() {
        return html`<p class="admin-status" data-status role="status" aria-live="polite"></p>`;
    }

    async load(operation) {
        try {
            await operation();
            if (this.isConnected) this.render();
        } catch (error) {
            if (this.isConnected) {
                this.render(html`<section class="content-page admin-page"><div class="admin-panel" role="alert">
                    <h1>Unable to load this page</h1><p>${error.message}</p>
                    <button class="admin-button" @click=${() => window.location.reload()}>Try again</button>
                </div></section>`);
            }
        }
    }
}
