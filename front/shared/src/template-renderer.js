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

export class TemplateRenderer extends HTMLElement {
    connectedCallback() {
        if (!this.noShadow) {
            this.attachShadow({ mode: "open" });
        }
        this.render();
    }

    render(template) {
        render(template || this.template, this);
    }
}
