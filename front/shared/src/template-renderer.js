import { html as exportedHtml, render } from "lit-html";
import { unsafeHTML as exportedUnsafeHTML } from "lit-html/directives/unsafe-html.js";

export const html = exportedHtml;
export const unsafeHTML = exportedUnsafeHTML;

function isSafeAttribute(name, value) {
    if (!name) {
        return false;
    }

    const lowerName = name.toLowerCase();
    if (lowerName.startsWith("on")) {
        return false;
    }

    if (lowerName === "src" || lowerName === "href") {
        const normalized = String(value || "").trim().toLowerCase();
        return !normalized.startsWith("javascript:");
    }

    return true;
}

function isSafeTag(tagName) {
    const blockedTags = new Set(["SCRIPT", "IFRAME", "OBJECT", "EMBED"]);
    return !blockedTags.has(tagName);
}

export function sanitizeHtml(htmlContent) {
    const parser = new DOMParser();
    const doc = parser.parseFromString(htmlContent, "text/html");
    const elements = doc.body.querySelectorAll("*");

    for (const el of elements) {
        const attrs = el.attributes;
        for (let i = attrs.length - 1; i >= 0; i--) {
            const attr = attrs[i];
            if (!isSafeAttribute(attr.name, attr.value)) {
                el.removeAttributeNode(attr);
            }
        }

        if (!isSafeTag(el.tagName)) {
            el.remove();
        }
    }

    return doc.body.innerHTML;
}

/**
 * Sanitize HTML then wrap it with lit-html's unsafeHTML directive.
 * Use this instead of calling unsafeHTML() directly to prevent XSS.
 */
export function safeHTML(htmlContent) {
    return exportedUnsafeHTML(sanitizeHtml(htmlContent));
}

/**
 * Component loading states.
 * @readonly
 * @enum {string}
 */
export const LoadState = Object.freeze({
    IDLE: "idle",
    LOADING: "loading",
    ERROR: "error",
    DONE: "done",
});

/**
 * Base class for all web components.
 * Provides rendering, loading/error state management, and
 * an AbortController that is wired to the component lifecycle.
 */
export class TemplateRenderer extends HTMLElement {
    constructor() {
        super();
        /** @type {AbortController | null} */
        this._abortController = null;
        /** @type {string} */
        this._loadState = LoadState.IDLE;
        /** @type {string | null} */
        this._errorMessage = null;
    }

    /** AbortSignal tied to this component's lifecycle. */
    get signal() {
        return this._abortController?.signal;
    }

    /**
     * Set the loading state and optionally re-render.
     * @param {string} state - One of LoadState values.
     * @param {string} [errorMessage]
     */
    setLoadState(state, errorMessage = null) {
        this._loadState = state;
        this._errorMessage = errorMessage;
    }

    /** @returns {boolean} */
    get isLoading() {
        return this._loadState === LoadState.LOADING;
    }

    /** @returns {boolean} */
    get hasError() {
        return this._loadState === LoadState.ERROR;
    }

    /** Loading indicator fragment — override in subclasses for custom UI. */
    get loadingTemplate() {
        return html`<div class="loading-indicator" role="status" aria-live="polite">
            <span class="loading-spinner" aria-hidden="true"></span>
            <span>Loading...</span>
        </div>`;
    }

    /** Error indicator fragment — override in subclasses for custom UI. */
    get errorTemplate() {
        return html`<div class="error-indicator" role="alert">
            <p class="error-message">${this._errorMessage || "Something went wrong."}</p>
        </div>`;
    }

    connectedCallback() {
        this._abortController = new AbortController();
        if (!this.noShadow) {
            this.attachShadow({ mode: "open" });
        }
        this.render();
    }

    disconnectedCallback() {
        this._abortController?.abort();
        this._abortController = null;
    }

    render(template) {
        render(template || this.template, this);
    }
}
