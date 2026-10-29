# Core frontend modules

Shared modules live in `front/shared/src/` and are imported by both apps.
App-only modules live in `front/<app>/app/core/`. Copy and adapt the code
below when a module does not exist yet; when one exists, move it toward this
shape without breaking its tests.

## `core/component.js` (per app, or shared once both apps use it)

```js
import { html, render } from 'lit-html';

/**
 * Base class for every page and component.
 * - template(): return a lit-html TemplateResult
 * - setState(patch): merge state and re-render
 * - signal: AbortSignal aborted when the element leaves the DOM
 * Set `static shadow = true` to render into a shadow root that links the bundled CSS.
 */
export class BaseComponent extends HTMLElement {
  static shadow = false;

  constructor() {
    super();
    this._state = {};
    this._abortController = new AbortController();
    if (this.constructor.shadow) {
      this.attachShadow({ mode: 'open' });
      const link = document.createElement('link');
      link.rel = 'stylesheet';
      link.href = '/style.css';
      this.shadowRoot.appendChild(link);
    }
  }

  get root() {
    return this.shadowRoot ?? this;
  }

  get signal() {
    return this._abortController.signal;
  }

  setState(patch) {
    this._state = { ...this._state, ...patch };
    this._render();
    return this._state;
  }

  connectedCallback() {
    if (this._abortController.signal.aborted) {
      this._abortController = new AbortController();
    }
    this._render();
  }

  disconnectedCallback() {
    this._abortController.abort();
  }

  _render() {
    try {
      render(this.template(), this.root);
    } catch (err) {
      console.error(`Render error in <${this.tagName.toLowerCase()}>:`, err);
      render(html`<div class="error-boundary" role="alert">Something went wrong.</div>`, this.root);
    }
  }

  template() {
    throw new Error('template() must be implemented');
  }
}
```

Light DOM (the default) is simpler and uses the global stylesheet. Use
shadow DOM only for reusable widgets that need `<slot>` and encapsulation.

## Registration: `shared/src/component-registry.js` + `app/core/core.js`

```js
// shared/src/component-registry.js
export class ComponentRegistry {
  static register(components) {
    for (const { tagName, component } of components) {
      if (!customElements.get(tagName)) customElements.define(tagName, component);
    }
  }
}
```

Each app lists its components once in `app/core/core.js`:

```js
const components = [
  ...layoutComponents,
  { tagName: 'c-drag-drop', component: DragDropComponent },
  { tagName: 'p-projects', component: ProjectsComponent },
  { tagName: 'p-not-found', component: NotFoundComponent },
];
ComponentRegistry.register(components);
```

## Router: `shared/src/router.js`

The shared `Router` class already provides the contract below and has
tests (`router.test.js`). Keep it the only router; the per-app
`controller/router-outlet.js` should only bind it to `<router-outlet>`.

- Routes: `{ path: '/projects/:id', component: 'p-project', beforeEnter? }`
  plus a final `{ path: '*', component: 'p-not-found' }`.
- Paths are normalised (trailing slash removed); `:param` segments are
  decoded into `location.params`.
- `beforeEnter(context)` may return a path to redirect to (admin auth
  guard) or `true` to continue. It may be async.
- Link clicks are intercepted only for same-origin, left-click, unmodified,
  non-download, `_self` anchors; `#`, `mailto:`, and `tel:` are ignored.
- `navigate(path)` pushes history and re-matches; `popstate` re-matches.
- The outlet replaces its child with `document.createElement(route.component)`.
  Pages read `location.params` in `connectedCallback`.

## API base URL: `shared/src/api-config.js`

Resolution order: `window.__DJMX_API_BASE_URL__` → `<meta
name="djmx-api-base-url">` → build-time `import.meta.env.BACKEND_API_URL`
→ `''` (relative). Trailing slashes are trimmed. `apiUrl(path)` returns
absolute URLs unchanged and prefixes everything else.

## HTTP: `shared/src/http.js` (target shape)

The current `BaseHttp` class in `http-base.js` (with its GET cache and
`tokenProvider`) moves to this small functional client. Keep the TTL cache
for public GETs.

```js
import { apiUrl } from './api-config.js';

export class HttpError extends Error {
  constructor(status, statusText, body) {
    super(`${status} ${statusText}`);
    this.name = 'HttpError';
    this.status = status;
    this.body = body;   // { errors: { message: [...] } } from the API
  }
}

/** Create a client. Admin passes a tokenProvider; portfolio passes nothing. */
export function createHttp({ tokenProvider } = {}) {
  async function http(path, options = {}) {
    const method = String(options.method || 'GET').toUpperCase();
    const headers = { ...options.headers };
    const isForm = options.body instanceof FormData;
    if (options.body !== undefined && !isForm) headers['Content-Type'] = 'application/json';
    const token = await tokenProvider?.();
    if (token) headers.Authorization = `Bearer ${token}`;

    let response;
    try {
      response = await fetch(apiUrl(path), {
        ...options,
        method,
        headers,
        body: isForm || options.body === undefined ? options.body : JSON.stringify(options.body),
        credentials: 'same-origin',
      });
    } catch (err) {
      if (err.name === 'AbortError') throw err;
      throw new HttpError(0, 'Network Error', null);
    }
    if (!response.ok) {
      let body = null;
      try { body = await response.json(); } catch { /* not JSON */ }
      throw new HttpError(response.status, response.statusText, body);
    }
    if (response.status === 204) return null;
    const type = response.headers.get('content-type') || '';
    return type.includes('application/json') ? response.json() : null;
  }

  return {
    get: (path, signal) => http(path, { signal }),
    post: (path, body, signal) => http(path, { method: 'POST', body, signal }),
    put: (path, body, signal) => http(path, { method: 'PUT', body, signal }),
    patch: (path, body, signal) => http(path, { method: 'PATCH', body, signal }),
    del: (path, signal) => http(path, { method: 'DELETE', signal }),
  };
}

export function buildQuery(params = {}) {
  const qs = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null && v !== '') qs.set(k, String(v));
  }
  const s = qs.toString();
  return s ? `?${s}` : '';
}
```

For multipart uploads pass a `FormData` body; the browser sets the boundary.
Error messages for the UI come from `err.body?.errors?.message?.[0]`.

## Services: `app/services/<resource>-service.js`

```js
import { API_ROUTES } from '../../../shared/src/api-routes.js';
import { buildQuery } from '../../../shared/src/http.js';
import { api } from '../core/api.js';   // createHttp({ tokenProvider }) once per app

export const listProjects = (params, signal) =>
  api.get(`${API_ROUTES.PROJECTS_V2}${buildQuery(params)}`, signal);
export const getProject = (id, signal) => api.get(API_ROUTES.PROJECT_BY_ID(id), signal);
export const updateProject = (id, body, signal) => api.put(API_ROUTES.PROJECT_BY_ID(id), body, signal);
```

Every API path is declared once in `shared/src/api-routes.js`.

## Store: `shared/src/store.js`

```js
export class Store {
  constructor(initial = {}) { this._state = { ...initial }; this._listeners = new Set(); }
  getState() { return { ...this._state }; }
  setState(patch) { this._state = { ...this._state, ...patch }; this._notify(); }
  subscribe(cb) { this._listeners.add(cb); return () => this._listeners.delete(cb); }
  _notify() {
    const s = this.getState();
    for (const cb of this._listeners) { try { cb(s); } catch (e) { console.error(e); } }
  }
}
```

Each app creates its own `appStore = new Store({ user: null, toasts: [] })`.
Components that subscribe call the returned unsubscribe function in
`disconnectedCallback`.

## `app/index.js`

```js
import './core/core.js';                 // registers every component
import { startAuth } from './auth/authentication.js';   // admin only
await startAuth();                       // admin: login before the outlet renders
document.body.querySelector('router-outlet')?.removeAttribute('hidden');
```

The portfolio skips the auth step.
