import assert from "node:assert/strict";
import test from "node:test";

import { Router } from "./router.js";

function createMockBrowserEnvironment(startPath = "/") {
    const listeners = {
        window: new Map(),
        document: new Map(),
    };

    const location = {
        origin: "http://localhost:3000",
        pathname: startPath,
        search: "",
        hash: "",
    };

    const windowMock = {
        location,
        history: {
            pushState(_state, _title, nextPath) {
                const next = new URL(nextPath, location.origin);
                location.pathname = next.pathname;
                location.search = next.search;
                location.hash = next.hash;
            },
        },
        addEventListener(type, handler) {
            listeners.window.set(type, handler);
        },
        removeEventListener(type) {
            listeners.window.delete(type);
        },
    };

    const documentMock = {
        addEventListener(type, handler) {
            listeners.document.set(type, handler);
        },
        removeEventListener(type) {
            listeners.document.delete(type);
        },
        createElement(tagName) {
            return { tagName };
        },
    };

    const outlet = {
        lastRendered: null,
        replaceChildren(child) {
            this.lastRendered = child;
        },
    };

    const invokeDocumentClick = async (anchor) => {
        const clickHandler = listeners.document.get("click");
        if (!clickHandler) {
            throw new Error("click listener not registered");
        }

        const event = {
            defaultPrevented: false,
            metaKey: false,
            ctrlKey: false,
            shiftKey: false,
            altKey: false,
            button: 0,
            preventDefault() {
                this.defaultPrevented = true;
            },
            target: {
                closest(selector) {
                    if (selector === "a[href]") {
                        return anchor;
                    }
                    return null;
                },
            },
        };

        await clickHandler(event);
        return event;
    };

    return {
        windowMock,
        documentMock,
        outlet,
        listeners,
        invokeDocumentClick,
    };
}

function makeAnchor(href, { target = "", download = false } = {}) {
    return {
        target,
        hasAttribute(name) {
            return name === "download" ? download : false;
        },
        getAttribute(name) {
            if (name === "href") {
                return href;
            }
            return "";
        },
    };
}

test("router resolves static and param routes", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([
        { path: "/", component: "p-home" },
        { path: "/projects/:id", component: "p-project" },
    ]);

    const result = await router.resolve("/projects/42");

    assert.equal(result.params.id, "42");
    assert.equal(env.outlet.lastRendered.tagName, "p-project");

    router.dispose();
    Router.activeRouter = null;
});

test("router navigate updates location and renders", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([
        { path: "/", component: "p-home" },
        { path: "/works/:id", component: "p-work" },
    ]);

    await router.navigate("/works/alpha");

    assert.equal(env.windowMock.location.pathname, "/works/alpha");
    assert.equal(env.outlet.lastRendered.tagName, "p-work");

    router.dispose();
    Router.activeRouter = null;
});

test("Router.go delegates to active router", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([
        { path: "/", component: "p-home" },
        { path: "/about", component: "p-about" },
    ]);

    await Router.go("/about");

    assert.equal(env.windowMock.location.pathname, "/about");
    assert.equal(env.outlet.lastRendered.tagName, "p-about");

    router.dispose();
    Router.activeRouter = null;
});

test("router intercepts internal anchor clicks", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([
        { path: "/", component: "p-home" },
        { path: "/contact", component: "p-contact" },
    ]);

    const event = await env.invokeDocumentClick(makeAnchor("/contact"));

    assert.equal(event.defaultPrevented, true);
    assert.equal(env.windowMock.location.pathname, "/contact");
    assert.equal(env.outlet.lastRendered.tagName, "p-contact");

    router.dispose();
    Router.activeRouter = null;
});

test("dispose removes listeners and active router", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([{ path: "/", component: "p-home" }]);

    assert.ok(env.listeners.window.has("popstate"));
    assert.ok(env.listeners.document.has("click"));

    router.dispose();

    assert.equal(Router.activeRouter, null);
    assert.equal(env.listeners.window.has("popstate"), false);
    assert.equal(env.listeners.document.has("click"), false);
});

test("router resolves wildcard route for unknown paths", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([
        { path: "/", component: "p-home" },
        { path: "*", component: "p-not-found" },
    ]);

    const result = await router.resolve("/does-not-exist");

    assert.equal(result.route.path, "*");
    assert.equal(env.outlet.lastRendered.tagName, "p-not-found");

    router.dispose();
    Router.activeRouter = null;
});

test("router guard can redirect before rendering", async () => {
    const env = createMockBrowserEnvironment("/");
    globalThis.window = env.windowMock;
    globalThis.document = env.documentMock;

    const router = new Router(env.outlet);
    await router.setRoutes([
        { path: "/", component: "p-home" },
        {
            path: "/admin",
            component: "p-admin",
            beforeEnter: () => "/",
        },
    ]);

    const result = await router.resolve("/admin");

    assert.equal(result.route.path, "/");
    assert.equal(env.windowMock.location.pathname, "/");
    assert.equal(env.outlet.lastRendered.tagName, "p-home");

    router.dispose();
    Router.activeRouter = null;
});
