# Frontend Commands

From this folder, use:

- `npm run dev` to start both admin and portfolio dev servers.
- `npm run build` to build both apps.
- `npm run build:admin` to build only admin.
- `npm run build:portfolio` to build only portfolio.

Default dev URLs:

- admin: http://localhost:3008
- portfolio: http://localhost:3009

## Public portfolio

The portfolio keeps its full-screen indexed menu and animated Works list.
Home uses the introduction and featured project saved in the admin's **Home**
section. The introduction is optional plain text (up to 500 characters).
Choose a published, non-adult project with an image cover, or **Remove selection** to keep
the abstract visual. There is no automatic replacement of the featured project.
A chosen project stops appearing if it becomes private/adult or loses its cover;
deleting it clears the selection. Other eligible projects remain in Selected works.
About and Contact use the content saved in admin.

Adult Works previews remain pixelated and blurred throughout the hover animation.
Opening an adult project requires choosing **View project**; **Hide adult content**
hides its description and media again. This is a presentation preference, not
access control: published media remain publicly addressable.

Project galleries preserve media order and image proportions. The first image
opens the gallery on its own; subsequent adjacent portrait/square images pair
on desktop, wide images span the gallery, and unpaired portraits are centered.
The layout adapts as image dimensions become available. Mobile uses one column.

Project images open in a modal viewer with previous/next buttons, arrow-key
navigation, Escape to close, and focus returned to the original image button.
The menu supports Escape and keyboard focus containment. Touch devices use the
same project links without the mouse-following preview; reduced-motion preferences
disable the animated tracking.

The browser suite includes deterministic public-page fixtures for pagination,
adult previews, the image viewer, empty/error states, and narrow-screen navigation,
alongside the admin's real API/persistence checks.

## Admin workflows

- **Home**: edit the short introduction, select the featured project, and choose
  **Save home**. Clear the text to hide it. Settings are stored independently
  of the About biography and existing project spotlights.
  Search projects by title or client (case/accent insensitive), identify them by
  their covers, and select a card. Results appear in groups of 12; the selected
  project stays visible while searching. The admin fetches the project catalogue
  once per visit and filters locally. Public Works loads six projects at a time
  from the API; **Load More** appends the next page.

- **Projects**: choose **New project**, enter a title, and create the draft.
  Edit its description and metadata, import media, choose a cover, then save.
  **Publish on portfolio** controls public visibility; new projects are private.
  Removing a project or a media file asks for confirmation.
- **About me**: edit the name and biography, save, and import or remove the
  profile photo separately. Media actions preserve text being edited.
- **Contacts**: add email, phone, and website/social-link rows, edit the introductory
  text, and choose **Save contacts**. Removing a row is applied when
  saving. Saved links appear on the public Contact page. Older description-only
  contact content remains supported; no database migration is needed.
- **Mixed media**: photos and MP4/WebM videos can share a project. Set each media's
  caption, photo alternative text, order number and section (**Main gallery** or
  **Behind the scenes**), then choose **Save media details**. Set the optional
  Behind the scenes description with **Save project**. Select an uploaded project
  image as a video's poster. Removing that image removes the poster from playback.
- Direct uploads allow 32 MiB per image and 2 GiB per video, with 8 MiB parts.
  Upload progress, cancellation and retries are supported. Retry retains completed
  parts while the page remains open; leaving/reloading starts a new upload.
  The legacy multipart endpoint retains its 50 MiB request limit.
  Save buttons show progress and success/error feedback; failed actions can be retried.

`cargo run -p dev-server -- test --browser` tests creation, validation, error recovery,
publishing, media, profile, contacts, Home settings, deep links, reloads, and mobile
navigation on the local stack. It removes its test projects and restores the
profile, contact, and Home data.

Home settings require database migration **V8**. The local stack applies it on
startup; deployed installations must apply migrations before serving the updated
backend. `GET /api/home/v1` exposes the public settings; administrator credentials
are required for `GET` and `PUT /api/home/v1/settings`.

## Direct media storage

Apply migration **V9** before serving the updated API. The admin authenticates
through the normal OIDC flow; the API creates a project-scoped upload session,
signs each part and checks the storage-reported part sizes before completion.
File bytes go from the browser directly to RustFS/S3, never through the API.
No storage credentials or application bearer tokens are sent with part uploads.

`STORAGE_ENDPOINT` must be reachable from both the API and browsers, using HTTPS
when the website uses HTTPS. Presigned URLs must retain the host/path/query used
for signing; rewriting an internal hostname after signing invalidates them.
The storage endpoint must support multipart operations, HEAD and ranged GET
(`206 Partial Content`) for seeking during video playback.

Provision the bucket with CORS for the deployed admin/portfolio origins. For example:

```json
{
  "CORSRules": [{
    "AllowedOrigins": ["https://admin.example.com", "https://portfolio.example.com"],
    "AllowedMethods": ["PUT", "GET", "HEAD"],
    "AllowedHeaders": ["*"],
    "ExposeHeaders": ["ETag", "Accept-Ranges", "Content-Range"],
    "MaxAgeSeconds": 3600
  }]
}
```

The local stack configures CORS on its own bucket. Production provisioning must
configure it separately; API startup does not change a production bucket's policy.
Configure an **AbortIncompleteMultipartUpload** lifecycle rule (for example after
one day) to reclaim abandoned parts even when no application traffic is running.
Sessions expire after 24 hours; part URLs expire after 15 minutes and Retry gets
fresh URLs. New uploads also perform bounded cleanup of expired pending sessions.
Completed media use the existing expiring GET URLs. The public player renews the
URL on an expired-link failure, preserves its playback position and offers Retry
if renewal/playback still fails. Format/codec compatibility remains the editor's
responsibility: there is no transcoder or automatic video poster generation.

New authenticated endpoints under `/api/portfolio/v1/projects/{id}`:

- `POST /uploads`: create a session with `fileName`, `mimeType`, `size`.
- `POST /uploads/{session}/parts/{number}`: obtain a temporary PUT URL.
- `POST /uploads/{session}/complete`: verify and attach the media, idempotently.
- `POST /uploads/{session}/abort`: discard unfinished parts.
- `PUT /media/{content}`: save `caption`, `alt`, `position`, `behindScenes`, `posterId`.

Legacy media default to the main gallery with empty captions/alternative text;
their upload order is preserved. Video posters must be images from the same project.
Editorial fields live in each media's `content.details` JSON; the optional section
description is stored in `project.description.behindScenes` as a Quill delta.

## Sign in with OpenID Connect WASI Hub

Copy `front/.env.template` to `front/.env` and configure `OIDC_HUB_URL`,
`OIDC_REALM`, and `OIDC_CLIENT_ID` before building or starting the admin UI.
For deployment, set the public provider URL, realm, and client; no provider
checkout is required. The local-development defaults target
`http://localhost:3000/realms/portfolio` and the public `portfolio` client.
`keycloak-js` remains the browser adapter, using the Hub's
Keycloak-compatible realm endpoints, Authorization Code with S256 PKCE, and
the `openid profile roles` scopes. The Keycloak session iframe is disabled
because the Hub does not provide that endpoint.

Register the admin origin's root URL (for example `https://admin.example.com/`) as
the exact redirect URI. Sign-in always returns there; the app restores the
requested page, query, and fragment from tab-local session storage before starting
its router. This also supports direct links and page reloads on nested routes.
The Hub does not treat `/*` as a redirect wildcard.

The esbuild plugin `build/keycloak-hub-plugin.js` adapts Keycloak JS 26's
refresh-token metadata parsing to the Hub's opaque refresh tokens. It preserves
the original refresh token and access/ID-token parsing and nonce checks. The
build fails if the upstream code changes, so dependency upgrades require reviewing
this compatibility patch. Both build and watch use the same plugin.

Every admin API request obtains a current access token and sends it as
`Authorization: Bearer ...`, including multipart uploads and reads. The public
portfolio uses anonymous requests. Existing `KEYCLOAK_URL`, `KEYCLOAK_REALM`,
and `KEYCLOAK_CLIENT_ID` build variables remain fallback aliases; the `OIDC_*`
variables take priority. Never put a client secret in the frontend.

See the repository README's **Security** section for realm, audience, role,
callback, CORS, and backend setup. Restart dev builds after changing the variables.

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

## Dependency pins

Admin intentionally pins Quill to `2.0.2`: version `2.0.3` is affected by
[GHSA-v3m3-f69x-jf25](https://github.com/advisories/GHSA-v3m3-f69x-jf25)
(XSS in HTML export). Recheck the advisory and `npm audit` before updating this pin.
