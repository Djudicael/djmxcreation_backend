import assert from "node:assert/strict";
import path from "node:path";

export async function runMediaWorkflows({ page, state, authorization }) {
    const base = "/api/portfolio/v1/projects";
    const api = async (method, resource, data) => {
        const response = await page.request[method](state.urls.admin + resource, { headers: { Authorization: authorization }, data });
        assert.ok(response.ok(), `Media API HTTP ${response.status()}`);
        return response;
    };
    const fixture = await (await api("post", base, { title: "Mixed media validation" })).json();
    const project = `${base}/${fixture.id}`;
    const preview = await page.context().newPage();
    const status = text => page.locator("[data-status]").filter({ hasText: text }).waitFor();
    const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aD3sAAAAASUVORK5CYII=", "base64");
    let failParts = true;
    const partCounts = new Map();
    const partRoute = async route => {
        const request = route.request();
        const number = new URL(request.url()).searchParams.get("partNumber");
        if (request.method() === "PUT" && number) {
            partCounts.set(number, (partCounts.get(number) || 0) + 1);
            if (number === "2" && failParts) return route.fulfill({ status: 503, headers: { "access-control-allow-origin": state.urls.admin } });
        }
        return route.continue();
    };
    try {
        await page.goto(state.urls.admin + `/projects/${fixture.id}`);
        await page.locator("[data-scenes-editor] .ql-editor").waitFor();
        const video = await page.evaluate(async () => {
            const canvas = document.createElement("canvas"); canvas.width = 160; canvas.height = 90;
            const stream = canvas.captureStream(10);
            const recorder = new MediaRecorder(stream, { mimeType: "video/webm" });
            const chunks = [];
            recorder.ondataavailable = event => chunks.push(event.data);
            const done = new Promise(resolve => { recorder.onstop = resolve; });
            recorder.start();
            const context = canvas.getContext("2d");
            for (let i = 0; i < 5; i++) { context.fillStyle = i % 2 ? "#245984" : "#14756b"; context.fillRect(0, 0, 160, 90); await new Promise(resolve => setTimeout(resolve, 100)); }
            recorder.stop(); await done; stream.getTracks().forEach(track => track.stop());
            return Array.from(new Uint8Array(await new Blob(chunks, { type: "video/webm" }).arrayBuffer()));
        });
        await page.locator("[data-editor] .ql-editor").fill("A finished series.");
        await page.locator("[data-scenes-editor] .ql-editor").fill("Lighting and camera setup.");
        await page.route("**/*uploadId*", partRoute);
        await page.locator("[name=files]").setInputFiles([
            { name: "large-photo.png", mimeType: "image/png", buffer: Buffer.concat([png, Buffer.alloc(9 * 1024 * 1024)]) },
            { name: "poster.png", mimeType: "image/png", buffer: png },
            { name: "on-set.webm", mimeType: "video/webm", buffer: Buffer.from(video) },
        ]);
        await page.getByRole("button", { name: "Upload files", exact: true }).click();
        await page.getByRole("button", { name: "Retry upload", exact: true }).waitFor();
        assert.equal(partCounts.get("1"), 1);
        assert.equal(partCounts.get("2"), 3);
        failParts = false;
        await page.getByRole("button", { name: "Retry upload", exact: true }).click();
        await status("Files uploaded");
        assert.equal(partCounts.get("1"), 3, "Retry must not resend the completed first part of the large image");
        await page.unroute("**/*uploadId*", partRoute);
        console.log("Media workflow: multipart retry passed without resending completed parts");
        assert.equal(await page.locator(".admin-media-card").count(), 3);
        assert.match(await page.locator("[data-editor] .ql-editor").innerText(), /finished series/);
        assert.match(await page.locator("[data-scenes-editor] .ql-editor").innerText(), /camera setup/);
        const uploaded = await (await api("get", project)).json();
        const clip = uploaded.contents.find(item => item.mimeType === "video/webm");
        const images = uploaded.contents.filter(item => item.mimeType === "image/png");
        const card = id => page.locator(`[data-content-id="${id}"]`);
        await card(images[0].id).getByLabel("Caption", { exact: true }).fill("Final photograph");
        await card(images[0].id).getByLabel("Alternative text").fill("A photograph from the finished series");
        await card(images[0].id).getByLabel("Order", { exact: true }).fill("2");
        await card(images[0].id).getByRole("button", { name: "Save media details" }).click(); await status("Media details saved");
        await card(images[1].id).getByLabel("Section", { exact: true }).selectOption("scenes");
        await card(images[1].id).getByRole("button", { name: "Save media details" }).click(); await status("Media details saved");
        await card(clip.id).getByLabel("Caption", { exact: true }).fill("Setting up the lights");
        await card(clip.id).getByLabel("Section", { exact: true }).selectOption("scenes");
        await card(clip.id).getByLabel("Video poster").selectOption(images[1].id);
        await card(clip.id).getByRole("button", { name: "Save media details" }).click(); await status("Media details saved");
        await page.getByLabel("Publish on portfolio").check();
        await page.getByRole("button", { name: "Save project", exact: true }).click(); await status("Project saved");
        await page.reload(); await card(clip.id).getByLabel("Video poster").waitFor();
        assert.equal(await card(clip.id).getByLabel("Video poster").inputValue(), images[1].id);
        assert.match(await page.locator("[data-scenes-editor] .ql-editor").innerText(), /camera setup/);
        console.log("Media workflow: descriptions, media settings and reload passed");
        let releasePart, partStarted;
        const held = new Promise(resolve => { releasePart = resolve; });
        const started = new Promise(resolve => { partStarted = resolve; });
        const holdPart = async route => {
            if (route.request().method() !== "PUT") return route.continue();
            partStarted(); await held; await route.abort().catch(() => {});
        };
        await page.route("**/*uploadId*", holdPart);
        try {
            await page.locator("[name=files]").setInputFiles({ name: "cancelled.png", mimeType: "image/png", buffer: png });
            await page.getByRole("button", { name: "Upload files", exact: true }).click();
            await started;
            await page.getByRole("button", { name: "Cancel upload", exact: true }).click();
            await page.getByText("Upload cancelled.", { exact: true }).waitFor();
            assert.equal((await (await api("get", project)).json()).contents.length, 3);
            console.log("Media workflow: active upload cancellation passed");
        } finally { releasePart(); await page.unroute("**/*uploadId*", holdPart); }
        await preview.goto(state.urls.portfolio + `/works/${fixture.id}`);
        await preview.locator(".project-scenes video").waitFor();
        assert.match(await preview.locator(".scenes-description").innerText(), /camera setup/);
        assert.equal(await preview.locator(".project-gallery").first().locator("video").count(), 0);
        assert.equal(await preview.locator(".project-scenes video").getAttribute("preload"), "none");
        assert.ok(await preview.locator(".project-scenes video").getAttribute("poster"));
        await preview.getByAltText("A photograph from the finished series", { exact: true }).waitFor();
        assert.deepEqual(await preview.locator(".project-gallery").first().locator(".project-photo img").evaluateAll(images => images.map(image => image.alt)), ["A photograph from the finished series"]);
        assert.equal(await preview.locator(".project-scenes .project-photo").count(), 1);
        assert.equal(await preview.locator(".project-scenes video + figcaption").innerText(), "Setting up the lights");
        await preview.getByAltText("A photograph from the finished series", { exact: true }).click();
        await preview.getByRole("button", { name: "Next image", exact: true }).click();
        assert.equal(await preview.locator(".overlay-image").getAttribute("alt"), "Mixed media validation — image 2");
        await preview.getByRole("button", { name: /Close/ }).click();
        await preview.locator("video").evaluate(async video => { video.muted = true; await video.play(); });
        await preview.waitForFunction(() => document.querySelector("video")?.currentTime > 0);
        // Simulate an expired URL and verify a fresh API read replaces it.
        await preview.locator("video").evaluate(video => { video.pause(); video.src = "/expired-video.webm"; video.load(); });
        await preview.waitForFunction(() => document.querySelector("video")?.src.includes("X-Amz-"));
        await preview.setViewportSize({ width: 390, height: 844 });
        assert.ok(await preview.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
        await preview.screenshot({ path: path.join(path.dirname(process.env.DJMX_STACK_STATE), "mixed-media-mobile.png"), fullPage: true });
        console.log("PASS: direct multipart upload/retry, preserved descriptions, media captions/alt/order/sections, video poster/playback/URL renewal and mobile layout");
    } finally {
        await page.unroute("**/*uploadId*", partRoute);
        await preview.close();
        await api("delete", project);
        await page.goto(state.urls.admin + "/projects");
        await page.getByRole("heading", { name: "Projects", exact: true }).waitFor();
    }
}
