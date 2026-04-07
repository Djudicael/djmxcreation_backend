/**
 * API configuration resolver.
 * Reads the backend base URL from (in priority order):
 *   1. Runtime global `window.__DJMX_API_BASE_URL__`
 *   2. `<meta name="djmx-api-base-url" content="...">` tag
 *   3. `import.meta.env.BACKEND_API_URL` (set at build time)
 *   4. Empty string (relative paths)
 */
const RUNTIME_API_BASE_KEY = "__DJMX_API_BASE_URL__";

function trimTrailingSlash(value) {
    return value.replace(/\/+$/, "");
}

function getFromRuntimeGlobal() {
    if (typeof globalThis === "undefined") {
        return "";
    }

    const value = globalThis[RUNTIME_API_BASE_KEY];
    return typeof value === "string" ? value.trim() : "";
}

function getFromMetaTag() {
    if (typeof document === "undefined") {
        return "";
    }

    const tag = document.querySelector('meta[name="djmx-api-base-url"]');
    const content = tag?.getAttribute("content");
    return typeof content === "string" ? content.trim() : "";
}

function getFromImportMetaEnv() {
    try {
        const value = import.meta?.env?.BACKEND_API_URL;
        return typeof value === "string" ? value.trim() : "";
    } catch {
        return "";
    }
}

function resolveApiBaseUrl() {
    const configured = getFromRuntimeGlobal() || getFromMetaTag() || getFromImportMetaEnv();

    if (!configured) {
        // Empty string means API paths stay relative (e.g. "/api/..."), avoiding hardcoded hosts.
        return "";
    }

    return trimTrailingSlash(configured);
}

/** @type {{ rest_url: string }} */
export const config = {
    rest_url: resolveApiBaseUrl(),
};
