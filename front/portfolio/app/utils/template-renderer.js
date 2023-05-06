
import { html as exportedHtml, render } from 'lit-html';
import { unsafeHTML as exportedUnsafeHTML } from 'lit-html/directives/unsafe-html.js';
export const html = exportedHtml;
export const unsafeHTML = exportedUnsafeHTML;

export function sanitizeHtml(html) {
    // Use a DOM parser to parse the HTML and remove any unsafe tags or attributes
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    const elements = doc.body.querySelectorAll('*');
    for (const el of elements) {
        const attrs = el.attributes;
        for (let i = attrs.length - 1; i >= 0; i--) {
            const attr = attrs[i];
            if (!isSafeAttribute(attr.name, attr.value)) {
                el.removeAttributeNode(attr);
            }
        }
        if (!isSafeTag(el.tagName)) {
            el.parentNode.removeChild(el);
        }
    }
    return doc.body.innerHTML;
}

export class TemplateRenderer extends HTMLElement {

    connectedCallback() {
        if (!this.noShadow) {
            this.attachShadow({ mode: 'open' });
        }
        this.render();
    }

    render(template) {
        if (this.noShadow) {
            render(template || this.template, this);
        } else {
            render(template || this.template, this);
        }
    }
}


