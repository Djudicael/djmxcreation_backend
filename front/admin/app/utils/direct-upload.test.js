import test from "node:test";
import assert from "node:assert/strict";
import { DirectUpload } from "./direct-upload.js";

test("retry keeps uploaded parts and refreshes the URL for the failed part", async () => {
    const counts = new Map();
    let fail = true;
    const original = globalThis.XMLHttpRequest;
    globalThis.XMLHttpRequest = class {
        upload = {};
        open(method, url) { this.part = Number(url); }
        send() {
            counts.set(this.part, (counts.get(this.part) || 0) + 1);
            queueMicrotask(() => { this.status = this.part === 2 && fail ? 503 : 200; this.onload(); });
        }
    };
    const api = {
        beginUpload: async () => ({ id: "session", partSize: 4, partCount: 3 }),
        uploadPartUrl: async (_project, _session, part) => ({ url: String(part) }),
        completeUpload: async () => ({ id: "media" }),
    };
    try {
        const task = new DirectUpload(api, "project", new File(["1234567890"], "clip.webm", { type: "video/webm" }));
        await assert.rejects(task.send(() => {}), /Retry/);
        assert.equal(counts.get(1), 1);
        assert.equal(counts.get(2), 3);
        fail = false;
        assert.deepEqual(await task.send(() => {}), { id: "media" });
        assert.equal(counts.get(1), 1);
        assert.equal(counts.get(2), 4);
        assert.equal(counts.get(3), 1);
    } finally { globalThis.XMLHttpRequest = original; }
});

test("cancellation while creating a session aborts it before uploading bytes", async () => {
    let ready;
    let aborted = 0;
    const api = {
        beginUpload: () => new Promise(resolve => { ready = resolve; }),
        abortUpload: async () => { aborted++; },
    };
    const task = new DirectUpload(api, "project", new File(["a"], "clip.webm", { type: "video/webm" }));
    const sending = task.send(() => {});
    await task.abort();
    ready({ id: "session" });
    await assert.rejects(sending, /cancelled/);
    assert.equal(aborted, 1);
});
