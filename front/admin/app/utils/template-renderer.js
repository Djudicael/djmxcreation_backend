// import { html as exportedHtml, render } from 'lit-html';

const templateCache = new WeakMap();
const eventRegistry = new WeakMap();
const directives = new Map();

export function html(strings, ...values) {
  return { strings, values };
}

export function render(template, container) {
  if (
    !template ||
    typeof template !== "object" ||
    !Array.isArray(template.strings)
  ) {
    throw new Error("Invalid template");
  }

  let cachedTemplate = templateCache.get(container);
  if (!cachedTemplate) {
    cachedTemplate = document.createElement("template");
    templateCache.set(container, cachedTemplate);
  }

  let result = "";
  template.strings.forEach((string, i) => {
    result += string + (values[i] !== undefined ? processValue(values[i]) : "");
  });

  cachedTemplate.innerHTML = result;

  const fragment = cachedTemplate.content.cloneNode(true);
  processEvents(fragment, template.values);

  if (container.shadowRoot) {
    container.shadowRoot.innerHTML = "";
    container.shadowRoot.appendChild(fragment);
  } else {
    container.innerHTML = "";
    container.appendChild(fragment);
  }
}

function processValue(value) {
  if (typeof value === "function") {
    return value();
  } else if (Array.isArray(value)) {
    return value.map(processValue).join(" ");
  } else if (typeof value === "object" && value.__directive) {
    return "";
  }
  return String(value);
}

function processEvents(fragment, values) {
  fragment.querySelectorAll("[data-event]").forEach((el, index) => {
    const eventType = el.getAttribute("data-event");
    if (eventType && typeof values[index] === "function") {
      if (!eventRegistry.has(el)) {
        el.addEventListener(eventType, values[index]);
        eventRegistry.set(el, eventType);
      }
    }
  });
}

export function directive(fn) {
  return { __directive: true, fn };
}

directives.set("on", (event, handler) => {
  return directive((el) => {
    el.setAttribute("data-event", event);
    return handler;
  });
});

export function unsafeHTML(html) {
  return directive((el) => {
    el.innerHTML = html;
  });
}
export const html = exportedHtml;

export function sanitizeHtml(html) {
  // Use a DOM parser to parse the HTML and remove any unsafe tags or attributes
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, "text/html");
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
      el.parentNode.removeChild(el);
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
    if (this.noShadow) {
      render(template || this.template, this);
    } else {
      render(template || this.template, this);
    }
  }
}
