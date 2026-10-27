import test from "node:test";
import assert from "node:assert/strict";
import { BaseHttp } from "./http-base.js";

function createHttp() {
    return new BaseHttp({ rest_url: "http://localhost:8081" });
}

test("buildUrl concatenates base url and path", () => {
    const http = createHttp();

    assert.equal(http.buildUrl("/projects"), "http://localhost:8081/projects");
});

test("createJsonHeaders adds bearer token when provided", () => {
    const http = createHttp();

    const headers = http.createJsonHeaders("token-123");

    assert.equal(headers.Authorization, "Bearer token-123");
    assert.equal(headers["Content-Type"], "application/json");
});

test("doPatchMultipart rejects non FormData bodies", async () => {
    const http = createHttp();

    await assert.rejects(
        () => http.doPatchMultipart({ path: "/upload", body: {}, authToken: "a" }),
        /Invalid request body/
    );
});

test("doGet uses json headers and returns parsed payload", async () => {
    const http = createHttp();
    const calls = [];
    const originalFetch = global.fetch;

    global.fetch = async (url, options) => {
        calls.push({ url, options });
        return {
            ok: true,
            status: 200,
            statusText: "OK",
            json: async () => ({ ok: true }),
        };
    };

    try {
        const payload = await http.doGet("/health");

        assert.deepEqual(payload, { ok: true });
        assert.equal(calls.length, 1);
        assert.equal(calls[0].url, "http://localhost:8081/health");
        assert.equal(calls[0].options.headers.get("Accept"), "application/json, text/plain, */*");
    } finally {
        global.fetch = originalFetch;
    }
});

test("fresh reads bypass stored data and request no-store", async () => {
    const http = createHttp();
    const originalFetch = global.fetch;
    const calls = [];
    global.fetch = async (_url, options) => {
        calls.push(options);
        return { ok: true, json: async () => ({ version: calls.length }) };
    };
    try {
        assert.equal((await http.doGet("/home")).version, 1);
        assert.equal((await http.doGet("/home")).version, 1);
        assert.equal((await http.doGet("/home", { cache: false })).version, 2);
        assert.equal(calls[1].cache, "no-store");
        assert.equal((await http.doGet("/home", { cache: false })).version, 3);
    } finally { global.fetch = originalFetch; }
});

test("authenticated JSON, multipart and GET requests refresh and send the access token", async () => {
    let refreshes = 0;
    const http = new BaseHttp({ rest_url: "http://localhost:8081" }, {
        tokenProvider: async () => `fresh-${++refreshes}`,
    });
    const originalFetch = global.fetch;
    const calls = [];
    global.fetch = async (_url, options) => {
        calls.push(options);
        return { ok: true, json: async () => ({ ok: true }) };
    };
    try {
        await http.doGet("/projects");
        await http.doPost({ path: "/projects", body: {} });
        await http.doPutJson({ path: "/projects/1", body: {} });
        await http.doDeleteRaw({ path: "/projects/1" });
        await http.doPatchMultipart({ path: "/projects/1/contents", body: new FormData() });
        assert.equal(refreshes, 5);
        calls.forEach((call, i) => assert.equal(call.headers.get("Authorization"), `Bearer fresh-${i + 1}`));
        assert.equal(calls[4].headers.has("Content-Type"), false);
    } finally {
        global.fetch = originalFetch;
    }
});

test("refresh failure prevents an unauthenticated API request", async () => {
    const http = new BaseHttp({ rest_url: "http://localhost:8081" }, {
        tokenProvider: async () => { throw new Error("Session expired"); },
    });
    const originalFetch = global.fetch;
    let calls = 0;
    global.fetch = async () => { calls++; };
    try {
        await assert.rejects(http.doPost({ path: "/projects", body: {} }), /Session expired/);
        assert.equal(calls, 0);
    } finally {
        global.fetch = originalFetch;
    }
});
