
const marker = `{{djmx-html-${String(Math.random()).slice(2)}}}`;

function render(template, element) {
    const templateResult = template instanceof TemplateResult ? template : new TemplateResult(template);
    const instance = element.__litHtmlInstance;

    if (instance !== undefined && instance.template === templateResult.template) {
        instance.update(templateResult.values);
    } else {
        const fragment = templateResult.createFragment();
        templateResult.parts.forEach(part => part.commit());
        element.innerHTML = '';
        element.appendChild(fragment);
        element.__litHtmlInstance = new Instance(templateResult.template, templateResult.parts, element);
    }
}

class Instance {
    constructor(template, parts, element) {
        this.template = template;
        this.parts = parts;
        this.element = element;
    }

    update(values) {
        let i = 0;

        for (const part of this.parts) {
            if (part.setValue(values[i])) {
                part.commit();
            }
            i++;
        }
    }
}

class TemplateResult {
    constructor(template) {
        this.template = template;
        this.parts = [];
    }

    get strings() {
        return this.template.strings;
    }

    get values() {
        return this.template.values;
    }

    createFragment() {
        const fragment = document.createDocumentFragment();
        const strings = this.strings;
        const length = strings.length;

        for (let i = 0; i < length; i++) {
            const string = strings[i];
            const node = i === 0 ? document.createTextNode(string) : this.createMarker(string);
            fragment.appendChild(node);
        }

        return fragment;
    }

    createMarker(string) {
        return document.createComment(marker + string);
    }

    createParts(fragment) {
        const parts = [];
        const nodes = fragment.childNodes;
        const length = nodes.length;
        let index = 0;

        for (let i = 0; i < length; i++) {
            const node = nodes[i];

            if (node.nodeType === Node.TEXT_NODE) {
                const value = this.values[index];
                const part = new TextPart(node, value);
                parts.push(part);
                index++;
            } else if (node.nodeType === Node.COMMENT_NODE && node.textContent.startsWith(marker)) {
                const part = new NodePart(node);
                parts.push(part);
            } else {
                const children = this.createParts(node);
                parts.push(...children);
            }
        }

        return parts;
    }
}

class Part {
    constructor(node) {
        this.node = node;
        this.value = undefined;
    }

    setValue(value) {
        if (this.value !== value) {
            this.value = value;
            return true;
        }
        return false;
    }

    commit() {
        // Do nothing by default
    }
}

class TextPart extends Part {
    commit() {
        this.node.textContent = this.value;
    }
}

class NodePart extends Part {
    commit() {
        this.node.parentNode.replaceChild(this.value, this.node);
    }
}


export function html(strings, ...values) {
    return strings.reduce((result, string, i) => {
        const value = values[i] !== undefined ? values[i] : '';
        return result + string + value;
    }, '');
}

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


