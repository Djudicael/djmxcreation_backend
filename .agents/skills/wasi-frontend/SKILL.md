---
name: wasi-frontend
description: Build DJMXCreation's browser frontends (front/admin, front/portfolio, front/shared) with native Web Components (Custom Elements) and lit-html templates, bundled by esbuild. Use when adding pages, components, routes, API services, state, forms, direct uploads, responsive media, admin auth (OIDC PKCE via keycloak-js), styles, node tests, or Playwright tests. No React, Vue, Angular, Svelte, or the Lit framework (LitElement); only HTMLElement + lit-html.
---

# Frontends: native Web Components + lit-html

Two apps share one stack:

- **admin** (`front/admin`): authenticated back office for projects, media,
  home settings, about, and contact.
- **portfolio** (`front/portfolio`): public, anonymous site.
- **shared** (`front/shared/src`): router, HTTP base, API config and routes,
  component registry, safe template rendering, responsive images, media
  helpers. Code used by both apps lives here, once.

The stack:

- **Custom Elements** (`class X extends HTMLElement`) for every page and
  component, usually through `BaseComponent`.
- **`lit-html`** (`html`, `render`, and directives) for templating only. Do
  not use `LitElement` or `lit` decorators.
- **`esbuild`** for bundling: one JS bundle (`dist/js/index.js`), one CSS
  bundle, and an `index.html` copy, through the shared
  `front/build/build-app.js`.
- **No framework, router library, or state library.** Routing, HTTP, and
  state are small modules.
- The output is static files served in production by `frontend-server`
  (see `wasi-frontend-packaging`): a static shell plus API calls.

## Layout

```text
front/
├── package.json            # workspace scripts: dev, build (both apps), test (node --test)
├── build-all.js            # builds admin + portfolio
├── build/build-app.js      # shared esbuild build/watch: env loading, define, JS + CSS
├── build/keycloak-hub-plugin.js   # esbuild plugin wiring keycloak-js to the OIDC Hub
├── shared/src/             # router.js, http-base.js, api-config.js, api-routes.js,
│                           # component-registry.js, template-renderer.js, responsive-image.js, …
├── shared/style/           # shared CSS tokens and primitives
└── <app>/                  # admin | portfolio
    ├── package.json        # deps: lit-html, esbuild (+ keycloak-js, quill in admin)
    ├── buildJs.js          # one-line call to build-app
    ├── dev.js              # watch + static dev server with SPA fallback
    ├── index.html          # shell: layout elements + <router-outlet>
    ├── app/
    │   ├── index.js        # entry: registers components, starts auth (admin), then the router
    │   ├── core/           # core.js (component list → ComponentRegistry), component.js
    │   ├── auth/           # admin only: authentication.js (keycloak-js PKCE), guards
    │   ├── services/       # <resource>-service.js — the only place that calls http
    │   ├── components/
    │   │   ├── layout/     # c-header, c-sidebar, …
    │   │   └── ui/         # c-modal, c-drag-drop, c-toast, …
    │   ├── pages/          # <name>.component.js, tag p-<name>, one per route
    │   └── utils/          # validators, format, error-handler, direct-upload
    ├── style/              # plain CSS split by concern
    └── ressource/          # static images/icons copied as-is (name kept on purpose)
```

Browser (Playwright) workflows live outside `front/`, in
`scripts/*-workflows.mjs`, and run against the local stack.

The current apps still have `api/`, `http/`, `controller/`, `lib/`, and
`models/` folders. When you touch one, move it toward the layout above:
`api/` and `http/` → `services/` on top of `shared/src/http-base.js`;
`controller/router-outlet.js` → `shared/src/router.js`; `lib/` → `utils/`.

## Core patterns

Read [core modules](references/core-modules.md) before you create or change
the base component, router, HTTP client, services, or store. Summary:

- **`BaseComponent`** extends `HTMLElement`. It has `template()`,
  `setState(patch)` (merge + re-render), an `AbortController` exposed as
  `this.signal` (aborted on disconnect), and an error boundary in
  `_render()`. Light DOM by default; `static shadow = true` for encapsulated
  widgets.
- **Router** (`shared/src/router.js`): a `routes` table
  `{ path, component, beforeEnter? }` with `:param` segments, a `*` fallback,
  History API navigation with link-click interception, and `navigate(path)`.
  `beforeEnter` returns a redirect for guarded admin routes.
- **HTTP** (`shared/src/http-base.js`): resolves the base URL
  (`shared/src/api-config.js`), sends JSON, adds `Authorization: Bearer` when
  the app provides a token getter, throws `HttpError(status, statusText,
  body)`, and returns `null` on 204. GET responses can use the short TTL
  cache; invalidate it after writes.
- **Services**: `services/<resource>-service.js` export plain functions
  (`listProjects(params, signal)`). URL paths come from
  `shared/src/api-routes.js`. **Components never call `fetch` directly.**
- **Store**: an observable `Store` singleton for cross-page state (user,
  toasts). Page-local state stays in the component's `_state`.

## Writing a page

```js
import { html } from 'lit-html';
import { BaseComponent } from '../core/component.js';
import { listProjects } from '../services/project-service.js';
import { handleApiError } from '../utils/error-handler.js';

export class ProjectsComponent extends BaseComponent {
  constructor() {
    super();
    this._state = { projects: [], loading: false };
  }

  connectedCallback() {
    super.connectedCallback();
    this._load();
  }

  async _load() {
    this.setState({ loading: true });
    try {
      const data = await listProjects({ page: 1, size: 20 }, this.signal);
      this.setState({ projects: data.projects ?? [], loading: false });
    } catch (err) {
      if (err.name === 'AbortError') return;
      handleApiError(err, 'Failed to load projects');
      this.setState({ loading: false });
    }
  }

  template() {
    const { projects, loading } = this._state;
    if (loading && !projects.length) return html`<div role="status">Loading…</div>`;
    if (!projects.length) return html`<p role="status">No projects yet.</p>`;
    return html`
      <h1>Projects</h1>
      <ul>${projects.map((p) => html`<li><a href="/projects/${p.id}">${p.title}</a></li>`)}</ul>
    `;
  }
}
```

Then add `{ tagName: 'p-projects', component: ProjectsComponent }` to the
component list in `app/core/core.js` and the route to the app's routes table.

Rules:

- **Naming:** tag `c-<name>` for reusable components and `p-<name>` for route
  pages. Files are `<name>.component.js`. Class names are PascalCase.
- **Registration:** one place per app, `ComponentRegistry.register([...])` in
  `app/core/core.js`. Do not also call `customElements.define` in the file.
- **Events:** bind with `@click=${this._onSave}` and `.value=${…}`. If you
  attach listeners to `window`/`document`, remove them in
  `disconnectedCallback`.
- **Data flow:** parent → child through properties (`.items=${…}`) or
  attributes (`observedAttributes` + `attributeChangedCallback`). Child →
  parent through `CustomEvent('name', { detail, bubbles: true, composed: true })`.
- **Async:** pass `this.signal` to every service call and ignore
  `AbortError`.
- **Loading/error/empty:** every data page renders all three states with
  `role="status"` / `role="alert"`.
- **Security:** lit-html escapes interpolations; keep it that way. Rich text
  (Quill deltas) is rendered only through the shared `safeHTML` sanitiser in
  `shared/src/template-renderer.js`. No inline `<script>` or `on*`
  attributes, so the CSP can stay `script-src 'self'`. Tokens stay in memory
  (keycloak-js), never in `localStorage`.
- **Media:** use `shared/src/responsive-image.js` for project images
  (`srcset` from the optimised variants) and `shared/src/media-kind.js` to
  decide between image and video. Large files use
  `admin/app/utils/direct-upload.js` (presigned multipart; see
  `wasi-object-storage`).
- **Accessibility:** real `<button>`/`<a>` elements, labels on inputs, focus
  return after a modal closes, Escape closes overlays, a skip link in
  `index.html`, and respect for `prefers-reduced-motion`.
- **Text:** UI text is English. Do not translate it based on the
  conversation language.

## Configuration and auth

- API calls use **relative paths** (`/api/...`) by default, so one build works
  behind any reverse proxy. Override order (in `shared/src/api-config.js`):
  `window.__DJMX_API_BASE_URL__`, then `<meta name="djmx-api-base-url">`, then
  the build-time `BACKEND_API_URL` injected through esbuild `define`
  (`import.meta.env`).
- Build-time env loading order: `front/.env` → `front/.env.local` →
  `front/<app>/.env` → `front/<app>/.env.local` → process env. Commit only
  templates.
- The admin uses OIDC Authorization Code + **PKCE S256** through
  `keycloak-js`, pointed at the OpenID Connect provider (the OpenID Connect
  WASI Hub in development) by `front/build/keycloak-hub-plugin.js` and the
  `OIDC_*` build variables. Never put a client secret in the frontend. Restore
  the deep link after the redirect.
- The portfolio is anonymous: its HTTP client never sends a token.

## Build and styles

Read [build and tooling](references/build-and-tooling.md) for the build
script, dev server, CSP, and npm scripts. Summary:

- `npm --prefix front run build` builds both apps into `front/<app>/dist`
  (`index.html`, `js/index.js`, `style.css`, extra stylesheets, and copied
  `ressource/`).
- Plain CSS with custom properties for theme tokens (`front/shared/style`).
  No preprocessors or CSS frameworks.
- Pin and audit dependencies (`npm audit`). Record each intentional pin with
  its reason in `front/README.md`.

## Tests

- Pure modules (router, http base, api config, validators, media helpers,
  direct upload): `node --test` files next to the source
  (`router.test.js`), listed in the `test` script of `front/package.json`.
- Pages and flows: Playwright workflows in `scripts/<area>-workflows.mjs`
  (wired into `scripts/stack-browser.mjs`) against the local stack started by
  `dev-server start` (see `wasi-dev-stack`); run with
  `dev-server test --browser`. Test real persistence, deep links, reloads, and
  a narrow viewport. Workflows clean up what they create.

## Done when

- `npm --prefix front run build` succeeds with no warnings, and
  `npm --prefix front test` passes.
- The new page works on a deep link and after a reload (SPA fallback).
- There are no new runtime dependencies besides `lit-html` (and the existing
  `keycloak-js`/`quill` in admin); any exception is justified in
  `front/README.md`.
