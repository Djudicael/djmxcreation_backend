# Frontend Commands

From this folder, use:

- `npm run dev` to start both admin and portfolio dev servers.
- `npm run build` to build both apps.
- `npm run build:admin` to build only admin.
- `npm run build:portfolio` to build only portfolio.

Default dev URLs:

- admin: http://localhost:3008
- portfolio: http://localhost:3009

## API Base URL Configuration

The frontend no longer hardcodes a backend host. API requests use relative paths by default (for example `/api/...`).

You can override the API base URL in this priority order:

1. Runtime global variable
2. HTML meta tag
3. Build/runtime env variable

### 1) Runtime global variable (highest priority)

Set this before loading your frontend bundle:

```html
<script>
	globalThis.__DJMX_API_BASE_URL__ = "https://api.example.com";
</script>
```

### 2) HTML meta tag

```html
<meta name="djmx-api-base-url" content="https://api.example.com" />
```

### 3) Environment variable

Set `BACKEND_API_URL` in your frontend build/runtime environment (used via `import.meta.env`).

Build/watch commands inject this value at bundle time through esbuild. The loader checks, in order, and then applies shell env as highest priority:

1. `front/.env`
2. `front/.env.local`
3. `front/admin/.env` or `front/portfolio/.env` (depending on app)
4. `front/admin/.env.local` or `front/portfolio/.env.local`
5. process environment (`BACKEND_API_URL`)

If none of these are set, the app keeps using relative API paths.
