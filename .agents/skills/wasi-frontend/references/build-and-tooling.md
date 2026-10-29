# Build and tooling

## Workspace `front/package.json`

```json
{
  "name": "djmx-front",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "node dev.js",
    "build": "node build-all.js",
    "build:admin": "npm --prefix admin run build",
    "build:portfolio": "npm --prefix portfolio run build",
    "test": "node --test shared/src admin/app portfolio/app"
  },
  "dependencies": {
    "lit-html": "^3.3.3"
  }
}
```

Each app has its own `package.json` and `package-lock.json` (`npm ci` per
directory). Check registry versions before you add or bump a dependency
(`npm view <pkg> version`) and run `npm audit` for every lockfile.

App `package.json` scripts:

```json
{
  "scripts": {
    "dev": "node dev.js",
    "build": "node buildJs.js",
    "build:watch": "nodemon --watch app --watch style buildJs.js",
    "e2e": "playwright test"
  }
}
```

## `front/build/build-app.js`

The shared builder exposes `buildApp(options)` and `watchApp(options)`:

- Loads env files in order: `front/.env`, `front/.env.local`,
  `front/<app>/.env`, `front/<app>/.env.local`, then `process.env`.
- Injects `import.meta.env` through esbuild `define` with only the public
  variables the browser needs (`BACKEND_API_URL`, the `OIDC_*` client
  settings for admin). Never inject secrets.
- Bundles `app/index.js` → `dist/js/index.js` (minified, sourcemaps only
  outside `NODE_ENV=production`).
- Bundles `style/style.css` → `dist/style.css`, plus any `extraStyles`.
- Copies `index.html` and `ressource/` into `dist/`.
- Applies `keycloak-hub-plugin.js`.

It runs from the app directory (`process.cwd()`). Each app's `buildJs.js` is a
short call:

```js
import { buildApp } from "../build/build-app.js";

buildApp({ extraStyles: { "preview.css": "style/preview.css" } }).catch((error) => {
    console.error(error);
    process.exit(1);
});
```

**Source maps (`*.map`) are never embedded** into the WASI frontend
component (the build script skips them).

Target improvement: emit content-hashed bundles (`js/index.<hash>.js`,
`style.<hash>.css`), rewrite `dist/index.html` to reference them, and let
`frontend-server` cache them as immutable.

## `dev.js`

Watch the build and serve `dist/` with SPA fallback:

```js
import express from 'express';
const app = express();
app.disable('x-powered-by');
app.use(express.static('dist'));
app.get('/{*path}', (_req, res) => res.sendFile('index.html', { root: 'dist' }));
app.listen(process.env.PORT || 3008);
```

For full-stack work, prefer `dev-server start --mode native` (API, provider,
proxy, and both apps on one origin) over a standalone dev server.

## `index.html` shell

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="djmx-api-base-url" content="">
  <meta http-equiv="Content-Security-Policy"
        content="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: STORAGE_ORIGIN; media-src 'self' STORAGE_ORIGIN; connect-src 'self' STORAGE_ORIGIN OIDC_ORIGIN; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none';">
  <link rel="stylesheet" href="/style.css">
  <title>DJMXCreation</title>
</head>
<body>
  <a href="#main-content" class="skip-link">Skip to content</a>
  <c-header></c-header>
  <main class="main">
    <router-outlet id="main-content"></router-outlet>
  </main>
  <script src="/js/index.js" type="module"></script>
</body>
</html>
```

Replace `STORAGE_ORIGIN` / `OIDC_ORIGIN` at build time from env. The
portfolio needs no `OIDC_ORIGIN`. Keep the CSP in `index.html` and in the
`frontend-server` response headers identical.

## CSS conventions

- Shared tokens and primitives in `front/shared/style/`; app files in
  `front/<app>/style/`, split by concern (layout, header, pages, forms).
- Put theme tokens in `:root { --color-…; --space-…; }`. Components use
  variables and never hard-code colours.
- BEM-like class names (`btn btn--primary btn--sm`).
- Mobile rules in the same file, under `@media (max-width: …)`.

## Playwright

Browser workflows are Node scripts in `scripts/` (`admin-workflows.mjs`,
`portfolio-workflows.mjs`, `media-workflows.mjs`, `home-workflows.mjs`),
driven by `scripts/stack-browser.mjs`. The CLI passes the runtime and
credentials files through `DJMX_STACK_STATE` and `DJMX_STACK_CREDENTIALS`;
the scripts read URLs and test users from them, run Chromium (Playwright
from the provider checkout's locked install), and clean up the data they
create.
Run them through `dev-server test --browser` so they hit a ready stack.
