import assert from "node:assert/strict";
import path from "node:path";

export async function runHomeWorkflows({ page, state, authorization }) {
    page.setDefaultTimeout(15000);
    const settingsPath = "/api/home/v1/settings";
    const projectPath = "/api/portfolio/v1/projects";
    const api = async (method, resource, options = {}) => {
        const response = await page.request[method](state.urls.admin + resource, { ...options, headers: { Authorization: authorization } });
        assert.ok(response.ok(), `${method} ${resource}: HTTP ${response.status()}`);
        return response;
    };
    const original = await (await api("get", settingsPath)).json();
    // Do not overwrite a stored unavailable selection that the API cannot restore.
    if (original.featuredProjectId) {
        const publicHome = await (await page.request.get(state.urls.portfolio + "/api/home/v1")).json();
        if (publicHome.featuredProjectId !== original.featuredProjectId) {
            console.log("Home write scenario skipped: preserving an existing unavailable selection.");
            return;
        }
    }
    const ids = [];
    const preview = await page.context().newPage();
    const status = text => page.locator("[data-status]").filter({ hasText: text }).waitFor();
    try {
        const fixture = await (await api("post", projectPath, { data: { title: "Home feature validation" } })).json();
        ids.push(fixture.id);
        const draft = await (await api("post", projectPath, { data: { title: "Home draft validation" } })).json();
        ids.push(draft.id);
        const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aD3sAAAAASUVORK5CYII=", "base64");
        const uploaded = await (await api("patch", `${projectPath}/${fixture.id}/contents`, { multipart: { photo: { name: "home-validation.png", mimeType: "image/png", buffer: png } } })).json();
        await api("put", `${projectPath}/${fixture.id}/thumbnails/${uploaded[0].id}`);
        const publish = adult => api("put", `${projectPath}/${fixture.id}`, { data: { metadata: { title: "Home feature validation" }, visible: true, adult } });
        await publish(false);
        await page.locator('a[href="/home"]').first().click();
        await page.getByLabel("Short introduction").waitFor();
        console.log("Home workflow: settings page loaded");
        assert.equal(await page.getByRole("button", { name: /Home draft validation/ }).count(), 0);
        const introduction = "I draw and photograph. <b>Plain text</b>";
        await page.getByLabel("Short introduction").fill(introduction);
        await page.getByRole("searchbox", { name: "Search projects" }).fill("Home feature validation");
        await page.getByRole("button", { name: /Home feature validation/ }).click();
        assert.equal(await page.getByLabel("Short introduction").inputValue(), introduction);
        const failSave = route => route.request().method() === "PUT"
            ? route.fulfill({ status: 503, json: {} }) : route.continue();
        await page.route(`**${settingsPath}`, failSave);
        await page.getByRole("button", { name: "Save home", exact: true }).click();
        await page.getByRole("alert").filter({ hasText: "failed" }).waitFor();
        assert.equal(await page.getByLabel("Short introduction").inputValue(), introduction);
        await page.unroute(`**${settingsPath}`, failSave);
        await page.getByRole("button", { name: "Save home", exact: true }).click();
        await status("Home saved");
        await page.reload();
        await page.getByLabel("Short introduction").waitFor();
        assert.equal(await page.getByLabel("Short introduction").inputValue(), introduction);
        await page.locator(".home-project-selection").getByText("Home feature validation", { exact: true }).waitFor();
        const saved = await (await api("get", settingsPath)).json();
        assert.deepEqual(saved, { introduction, featuredProjectId: fixture.id });
        console.log("Home workflow: save/retry and persistence passed");
        await preview.goto(state.urls.portfolio);
        await preview.locator(".featured-image").waitFor();
        assert.equal(await preview.locator(".featured-image").getAttribute("href"), `/works/${fixture.id}`);
        assert.equal(await preview.locator(".home-summary").innerText(), introduction);
        assert.equal(await preview.locator(".home-summary b").count(), 0);
        await page.screenshot({ path: path.join(path.dirname(process.env.DJMX_STACK_STATE), "admin-home-preview.png"), fullPage: true });

        // A later NSFW flag removes the hero, even when navigating back in the same tab.
        await publish(true);
        await preview.getByRole("link", { name: "About me", exact: true }).click();
        await preview.locator(".about-page").waitFor();
        await preview.locator(".wordmark").click();
        await preview.locator(".home-summary").waitFor();
        assert.equal(await preview.locator(".featured-image").count(), 0);
        assert.equal(await preview.locator(`a[href="/works/${fixture.id}"]`).count(), 0);
        await page.reload();
        await page.getByLabel("Short introduction").waitFor();
        await page.getByText("The selected project is no longer available on Home.", { exact: false }).waitFor();
        await page.getByRole("button", { name: "Save home", exact: true }).click();
        await page.getByRole("alert").filter({ hasText: "Choose an available project" }).waitFor();
        await page.getByRole("button", { name: "Remove selection", exact: true }).click();
        await page.getByLabel("Short introduction").fill("");
        await page.getByRole("button", { name: "Save home", exact: true }).click();
        await status("Home saved");
        await preview.reload();
        await preview.waitForFunction(() => document.querySelector("p-home")?._loadState === "done");
        await preview.locator(".visual-study").waitFor();
        assert.equal(await preview.locator(".home-summary").count(), 0);
        assert.equal(await preview.locator(".featured-image").count(), 0);
        await page.setViewportSize({ width: 390, height: 844 });
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
        await page.screenshot({ path: path.join(path.dirname(process.env.DJMX_STACK_STATE), "admin-home-mobile.png"), fullPage: true });
        // Exercise a large catalogue without creating hundreds of database records.
        const catalogue = Array.from({ length: 125 }, (_, index) => ({
            ...fixture, id: `catalogue-${index}`, visible: true, adult: false,
            metadata: { title: `Project ${index}`, client: index === 124 ? "Étoile Studio" : "Example" },
            thumbnail: { url: "data:image/png;base64," + png.toString("base64"), mimeType: "image/png" },
        }));
        const catalogueRoute = route => route.fulfill({ json: catalogue });
        await page.route(`**${projectPath}`, catalogueRoute);
        try {
            await page.reload();
            await page.getByRole("searchbox", { name: "Search projects" }).waitFor();
            assert.equal(await page.locator(".home-project-option").count(), 12);
            await page.getByRole("button", { name: "Show more projects" }).click();
            assert.equal(await page.locator(".home-project-option").count(), 24);
            await page.getByLabel("Short introduction").fill("Unsaved introduction");
            const search = page.getByRole("searchbox", { name: "Search projects" });
            await search.fill("etoile");
            assert.equal(await page.locator(".home-project-option").count(), 1);
            await search.press("Tab");
            await page.keyboard.press("Enter");
            assert.equal(await page.locator(".home-project-option[aria-pressed=true]").count(), 1);
            await search.fill("No such project");
            await page.getByText("No matching projects.", { exact: true }).waitFor();
            await page.locator(".home-project-selection").getByText("Project 124", { exact: true }).waitFor();
            assert.equal(await page.getByLabel("Short introduction").inputValue(), "Unsaved introduction");
            await search.fill("");
            assert.equal(await page.locator(".home-project-option").count(), 12);
            assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
            await page.screenshot({ path: path.join(path.dirname(process.env.DJMX_STACK_STATE), "admin-home-picker-mobile.png"), fullPage: true });
            await page.setViewportSize({ width: 1280, height: 900 });
            await page.screenshot({ path: path.join(path.dirname(process.env.DJMX_STACK_STATE), "admin-home-picker-desktop.png"), fullPage: true });
            console.log("PASS: 125-project picker, bounded results, title/client search, accents, keyboard selection, preserved edits and mobile layout");
        } finally { await page.unroute(`**${projectPath}`, catalogueRoute); }
        console.log("PASS: Home admin save/retry/reload, real public introduction/feature, plain text, NSFW revocation, clear settings, mobile layout");
    } finally {
        const results = await Promise.allSettled([
            api("put", settingsPath, { data: original }),
            ...ids.map(id => api("delete", `${projectPath}/${id}`)),
        ]);
        await preview.close();
        await page.setViewportSize({ width: 1280, height: 900 });
        assert.ok(results.every(result => result.status === "fulfilled"), "Could not clean up all Home fixtures");
    }
}
