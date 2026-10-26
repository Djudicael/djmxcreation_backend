import Keycloak from "keycloak-js";

// The Hub exposes realm endpoints compatible with the Keycloak browser adapter.
const env = import.meta.env || {};
const keycloak = new Keycloak({
    url: env.OIDC_HUB_URL || env.KEYCLOAK_URL || "http://localhost:3000",
    realm: env.OIDC_REALM || env.KEYCLOAK_REALM || "portfolio",
    clientId: env.OIDC_CLIENT_ID || env.KEYCLOAK_CLIENT_ID || "portfolio",
});

const returnPathKey = "djmx-admin:oidc-return-path";

function rememberRequestedPage() {
    const url = new URL(window.location.href);
    const callback = [url.search, url.hash.slice(1)].some(part => {
        const params = new URLSearchParams(part);
        return params.has("state") && (params.has("code") || params.has("error"));
    });
    if (!callback) {
        sessionStorage.setItem(returnPathKey, `${url.pathname}${url.search}${url.hash}`);
    }
}

function restoreRequestedPage() {
    const path = sessionStorage.getItem(returnPathKey);
    sessionStorage.removeItem(returnPathKey);
    if (path) {
        const url = new URL(path, window.location.origin);
        if (url.origin === window.location.origin) {
            window.history.replaceState(window.history.state, "", `${url.pathname}${url.search}${url.hash}`);
        }
    }
}

export async function initKeycloak(onAuthenticatedCallback) {
    try {
        rememberRequestedPage();
        const authenticated = await keycloak.init({
            // The Hub requires an exact registered callback, including on reload.
            redirectUri: `${window.location.origin}/`,
            pkceMethod: "S256",
            onLoad: "login-required",
            checkLoginIframe: false,
            scope: "openid profile roles",
        });
        if (authenticated) {
            restoreRequestedPage();
            onAuthenticatedCallback();
        }
    } catch (error) {
        console.error("OpenID Connect sign-in failed", error);
        document.body.textContent = "Unable to sign in. Please reload the page to try again.";
    }
}

export async function getAccessToken() {
    if (!keycloak.authenticated) {
        throw new Error("Authentication required");
    }
    try {
        await keycloak.updateToken(30);
    } catch (error) {
        keycloak.clearToken();
        throw new Error("Session expired. Please sign in again.", { cause: error });
    }
    if (!keycloak.token) {
        throw new Error("Authentication required");
    }
    return keycloak.token;
}

export function logout() {
    sessionStorage.removeItem(returnPathKey);
    return keycloak.logout({ redirectUri: `${window.location.origin}/` });
}
