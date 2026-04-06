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
            json: async () => ({ ok: true }),
        };
    };

    try {
        const payload = await http.doGet("/health");

        assert.deepEqual(payload, { ok: true });
        assert.equal(calls.length, 1);
        assert.equal(calls[0].url, "http://localhost:8081/health");
        assert.equal(calls[0].options.headers.Accept, "application/json, text/plain, */*");
    } finally {
        global.fetch = originalFetch;
    }
});
