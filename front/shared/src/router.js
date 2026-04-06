function normalizePath(pathname) {
    if (!pathname) {
        return "/";
    }

    if (pathname !== "/" && pathname.endsWith("/")) {
        return pathname.slice(0, -1);
    }

    return pathname;
}

function parsePathSegments(pathname) {
    return normalizePath(pathname)
        .split("/")
        .filter(Boolean);
}

function compileRoute(path) {
    const normalized = normalizePath(path);
    if (normalized === "*") {
        return {
            path: normalized,
            parts: [],
            keys: [],
            wildcard: true,
        };
    }

    const segments = parsePathSegments(normalized);
    const keys = [];

    const parts = segments.map((segment) => {
        if (segment.startsWith(":")) {
            const key = segment.slice(1);
            keys.push(key);
            return { type: "param", key };
        }

        return { type: "static", value: segment };
    });

    return {
        path: normalized,
        parts,
        keys,
    };
}

function matchRoute(route, pathname) {
    if (route.wildcard) {
        return {};
    }

    const segments = parsePathSegments(pathname);

    if (segments.length !== route.parts.length) {
        return null;
    }

    const params = {};

    for (let i = 0; i < route.parts.length; i += 1) {
        const part = route.parts[i];
        const value = segments[i];

        if (part.type === "static" && part.value !== value) {
            return null;
        }

        if (part.type === "param") {
            params[part.key] = decodeURIComponent(value);
        }
    }

    return params;
}

function isModifiedEvent(event) {
    return event.metaKey || event.ctrlKey || event.shiftKey || event.altKey;
}

function shouldHandleLinkClick(anchor, event) {
    if (!anchor || event.defaultPrevented || isModifiedEvent(event) || event.button !== 0) {
        return false;
    }

    if (anchor.target && anchor.target !== "_self") {
        return false;
    }

    if (anchor.hasAttribute("download")) {
        return false;
    }

    const href = anchor.getAttribute("href") || "";
    if (!href || href.startsWith("#") || href.startsWith("mailto:") || href.startsWith("tel:")) {
        return false;
    }

    const url = new URL(href, window.location.origin);
    if (url.origin !== window.location.origin) {
        return false;
    }

    return true;
}

export class Router {
    static activeRouter = null;

    constructor(outlet) {
        this.outlet = outlet;
        this.routes = [];
        this._onPopState = this._onPopState.bind(this);
        this._onDocumentClick = this._onDocumentClick.bind(this);
        Router.activeRouter = this;
    }

    setRoutes(routes) {
        this.routes = routes.map((route) => ({
            ...route,
            compiled: compileRoute(route.path),
        }));

        window.addEventListener("popstate", this._onPopState);
        document.addEventListener("click", this._onDocumentClick);

        return this.resolve(window.location.pathname);
    }

    dispose() {
        window.removeEventListener("popstate", this._onPopState);
        document.removeEventListener("click", this._onDocumentClick);

        if (Router.activeRouter === this) {
            Router.activeRouter = null;
        }
    }

    static go(path) {
        const router = Router.activeRouter;
        if (!router) {
            window.history.pushState({}, "", path);
            return;
        }

        router.navigate(path);
    }

    navigate(path) {
        const target = new URL(path, window.location.origin);
        const next = `${target.pathname}${target.search}${target.hash}`;

        if (next !== `${window.location.pathname}${window.location.search}${window.location.hash}`) {
            window.history.pushState({}, "", next);
        }

        return this.resolve(target.pathname);
    }

    async resolve(pathname) {
        const path = normalizePath(pathname || window.location.pathname);
        const wildcardRoute = this.routes.find((route) => route.path === "*");

        for (const route of this.routes) {
            if (route.path === "*") {
                continue;
            }

            const params = matchRoute(route.compiled, path);
            if (params) {
                const guardResult = await this._runGuard(route, {
                    pathname: path,
                    params,
                });
                if (guardResult.redirectTo) {
                    return this.navigate(guardResult.redirectTo);
                }
                if (guardResult.blocked) {
                    return {
                        pathname: path,
                        params,
                        route,
                    };
                }

                this._renderRoute(route.component);
                return {
                    pathname: path,
                    params,
                    route,
                };
            }
        }

        if (wildcardRoute) {
            const guardResult = await this._runGuard(wildcardRoute, {
                pathname: path,
                params: {},
            });
            if (guardResult.redirectTo) {
                return this.navigate(guardResult.redirectTo);
            }
            if (!guardResult.blocked) {
                this._renderRoute(wildcardRoute.component);
            }

            return {
                pathname: path,
                params: {},
                route: wildcardRoute,
            };
        }

        const fallback = this.routes.find((route) => route.path === "/") || this.routes[0];
        if (fallback) {
            this._renderRoute(fallback.component);
            return {
                pathname: path,
                params: {},
                route: fallback,
            };
        }

        return {
            pathname: path,
            params: {},
            route: null,
        };
    }

    _renderRoute(componentTag) {
        if (!this.outlet || !componentTag) {
            return;
        }

        this.outlet.replaceChildren(document.createElement(componentTag));
    }

    async _runGuard(route, context) {
        if (typeof route.beforeEnter !== "function") {
            return { blocked: false, redirectTo: null };
        }

        const result = await route.beforeEnter(context);

        if (typeof result === "string") {
            return { blocked: false, redirectTo: result };
        }

        if (result && typeof result === "object" && typeof result.redirectTo === "string") {
            return { blocked: false, redirectTo: result.redirectTo };
        }

        if (result === false) {
            return { blocked: true, redirectTo: null };
        }

        return { blocked: false, redirectTo: null };
    }

    _onPopState() {
        this.resolve(window.location.pathname);
    }

    async _onDocumentClick(event) {
        const anchor = event.target.closest("a[href]");
        if (!shouldHandleLinkClick(anchor, event)) {
            return;
        }

        const url = new URL(anchor.getAttribute("href"), window.location.origin);
        event.preventDefault();
        await this.navigate(`${url.pathname}${url.search}${url.hash}`);
    }
}
