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

export const config = {
    rest_url: resolveApiBaseUrl(),
};
