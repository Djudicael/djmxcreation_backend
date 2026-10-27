import assert from "node:assert/strict";
import path from "node:path";

// Test through the UI; use authenticated API calls only for snapshots/cleanup
// and independent persistence checks. Never print the bearer token.
export async function runAdminWorkflows({ page, state, authorization }) {
    const root = state.urls.admin;
    const stateDir = path.dirname(process.env.DJMX_STACK_STATE);
    const api = async (method, resource, options = {}) => {
        const response = await page.request[method](`${root}${resource}`, {
            ...options, headers: { Authorization: authorization },
        });
        assert.ok(response.ok(), `${method.toUpperCase()} ${resource}: HTTP ${response.status()}`);
        return response;
    };
    const aboutPath = "/api/about/v1/me";
    const contactPath = "/api/contact/v1/information";
    const projectPath = "/api/portfolio/v1/projects";
    const about = await (await api("get", aboutPath)).json();
    const contact = await (await api("get", contactPath)).json();
    const projects = new Set();
    const png = Buffer.from("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aD3sAAAAASUVORK5CYII=", "base64");
    let originalPhoto;
    if (about.photoUrl) {
        const image = await page.request.get(about.photoUrl);
        assert.ok(image.ok(), "Could not snapshot the original profile image");
        originalPhoto = { buffer: await image.body(), mimeType: image.headers()["content-type"] || "image/png" };
    }
    let photoChanged = false;
    const status = text => page.locator("[data-status]").filter({ hasText: text }).waitFor();
    const openAdmin = async route => {
        await page.goto(`${root}${route}`);
        await page.locator(".admin-page h1").waitFor();
    };
    try {
        await page.getByRole("button", { name: "+ New project", exact: true }).click();
        await page.getByRole("button", { name: "Cancel", exact: true }).click();
        assert.equal(await page.locator(".admin-create-form").count(), 0);
        await page.getByRole("button", { name: "+ New project", exact: true }).click();
        await page.getByRole("button", { name: "Create project", exact: true }).click();
        assert.equal(await page.locator("[name=title]:invalid").count(), 1, "A title is required");
        await page.getByLabel("Title", { exact: true }).fill("UI test project");
        const rejectCreation = route => route.request().method() === "POST"
            ? route.fulfill({ status: 503, contentType: "application/json", body: '{"error":"test unavailable"}' })
            : route.continue();
        await page.route(`**${projectPath}`, rejectCreation);
        await page.getByRole("button", { name: "Create project", exact: true }).click();
        await page.getByRole("alert").filter({ hasText: "failed" }).waitFor();
        await page.unroute(`**${projectPath}`, rejectCreation);
        const created = page.waitForResponse(response => response.url().endsWith(projectPath) && response.request().method() === "POST" && response.ok());
        await page.getByRole("button", { name: "Create project", exact: true }).click();
        const project = await (await created).json();
        projects.add(project.id);
        const projectUrl = `${root}/projects/${project.id}`;
        await page.waitForURL(projectUrl);
        await page.locator("[data-editor] .ql-editor").waitFor();
        console.log("Admin workflow: draft creation and retry passed");
        assert.equal((await page.request.get(`${root}${projectPath}/${project.id}`)).status(), 404, "Drafts are private");
        await page.getByLabel("Title", { exact: true }).fill("Published UI project");
        await page.getByLabel("Subtitle", { exact: true }).fill("Test subtitle");
        await page.getByLabel("Client", { exact: true }).fill("Demo client");
        await page.locator("[data-editor] .ql-editor").fill("Test description preserved during uploads.");
        await page.locator("input[name=files]").setInputFiles([
            { name: "validation-cover-one.png", mimeType: "image/png", buffer: Buffer.concat([png, Buffer.alloc(3 * 1024 * 1024)]) },
            { name: "validation-cover-two.png", mimeType: "image/png", buffer: png },
        ]);
        await page.getByRole("button", { name: "Upload files", exact: true }).click();
        await status("Files uploaded");
        console.log("Admin workflow: multipart upload above 2 MiB passed");
        assert.equal(await page.locator(".admin-media-card").count(), 2);
        await page.waitForFunction(() => {
            const images = [...document.querySelectorAll(".admin-media-card img")];
            return images.length === 2 && images.every(image => image.complete && image.naturalWidth > 0);
        });
        assert.equal(await page.getByLabel("Title", { exact: true }).inputValue(), "Published UI project");
        assert.match(await page.locator("[data-editor] .ql-editor").innerText(), /preserved during uploads/);
        for (const index of [0, 1]) {
            await page.getByRole("button", { name: "Set as cover", exact: true }).nth(index).click();
            await status("Cover updated");
        }
        await page.getByLabel("Publish on portfolio").check();
        await page.getByLabel("Adult content").check();
        await page.getByRole("button", { name: "Save project", exact: true }).click();
        await status("Project saved");
        await page.reload();
        await page.locator("[data-editor] .ql-editor").waitFor();
        assert.equal(await page.getByLabel("Title", { exact: true }).inputValue(), "Published UI project");
        assert.equal(await page.getByLabel("Publish on portfolio").isChecked(), true);
        assert.equal(await page.getByLabel("Adult content").isChecked(), true);
        const published = await (await page.request.get(`${root}${projectPath}/${project.id}`)).json();
        assert.equal(published.metadata.subTitle, "Test subtitle");
        assert.match(published.thumbnail.url, /validation-cover-two/);
        assert.equal(published.contents.length, 2);
        console.log("Admin workflow: save, publish, flags, cover replacement and reload passed");
        await page.screenshot({ path: path.join(stateDir, "admin-project-preview.png"), fullPage: true });
        const publicPage = await page.context().newPage();
        try {
            publicPage.on("pageerror", error => console.error("Public project error:", error.message));
            await publicPage.goto(`${state.urls.portfolio}/works/${project.id}`);
            await publicPage.getByText("Published UI project", { exact: true }).first().waitFor();
        } catch (error) {
            console.error("Public project page:", (await publicPage.locator("body").innerText()).slice(0, 1500));
            throw error;
        } finally { await publicPage.close(); }
        for (const remaining of [1, 0]) {
            const removedUrl = await page.locator(".admin-media-card img").first().getAttribute("src");
            page.once("dialog", dialog => dialog.accept());
            await page.getByRole("button", { name: "Delete media", exact: true }).first().click();
            await status("Media deleted");
            assert.equal(await page.locator(".admin-media-card").count(), remaining);
            assert.equal((await page.request.get(removedUrl)).status(), 404, "Deleted media must disappear from storage");
        }
        const emptied = await (await api("get", `${projectPath}/${project.id}`)).json();
        assert.equal(emptied.thumbnail, null);
        await page.getByLabel("Publish on portfolio").uncheck();
        await page.getByRole("button", { name: "Save project", exact: true }).click();
        await status("Project saved");
        assert.equal((await page.request.get(`${root}${projectPath}/${project.id}`)).status(), 404);
        await page.getByRole("link", { name: "← All projects", exact: true }).click();
        const card = page.locator(`[data-project-id="${project.id}"]`);
        page.once("dialog", dialog => dialog.accept());
        await card.getByRole("button", { name: "Delete", exact: true }).click();
        await status("Project deleted");
        assert.equal(await card.count(), 0);
        projects.delete(project.id);
        console.log("Admin workflow: public project view, media deletion, hide and project deletion passed");

        await page.getByRole("link", { name: /About me/ }).first().click();
        await page.locator("[data-editor] .ql-editor").waitFor();
        await page.getByLabel("First name", { exact: true }).fill("Test");
        await page.getByLabel("Last name", { exact: true }).fill("Profile");
        await page.locator("[data-editor] .ql-editor").fill("Test profile biography.");
        await page.getByRole("button", { name: "Save about me", exact: true }).click();
        await status("About me saved");
        await page.locator("input[name=photo]").setInputFiles({ name: "validation-profile.png", mimeType: "image/png", buffer: png });
        photoChanged = true;
        await page.getByRole("button", { name: "Upload photo", exact: true }).click();
        await status("Photo uploaded");
        await page.getByAltText("Current profile photo").waitFor();
        page.once("dialog", dialog => dialog.accept());
        await page.getByRole("button", { name: "Delete photo", exact: true }).click();
        await status("Photo deleted");
        assert.equal(await page.getByAltText("Current profile photo").count(), 0);
        await page.reload();
        await page.locator("[data-editor] .ql-editor").waitFor();
        assert.equal(await page.getByLabel("First name", { exact: true }).inputValue(), "Test");
        assert.match(await page.locator("[data-editor] .ql-editor").innerText(), /Test profile biography/);
        const profilePreview = await page.context().newPage();
        try {
            await profilePreview.goto(`${state.urls.portfolio}/about`);
            await profilePreview.getByRole("heading", { name: "Test Profile", exact: true }).waitFor();
            await profilePreview.getByText("Test profile biography.", { exact: true }).waitFor();
            assert.equal(await profilePreview.getByAltText("Profile photo").count(), 0);
        } finally { await profilePreview.close(); }
        console.log("Admin workflow: profile text, photo upload/delete and persistence passed");

        await page.getByRole("link", { name: /Contacts/ }).first().click();
        await page.locator("[data-editor] .ql-editor").waitFor();
        await page.getByRole("button", { name: "+ Add contact", exact: true }).click();
        let row = page.locator(".admin-contact-row").last();
        await row.getByLabel("Label", { exact: true }).fill("Test email");
        await row.getByLabel("Contact detail", { exact: true }).fill("validation@example.com");
        await page.getByRole("button", { name: "+ Add contact", exact: true }).click();
        row = page.locator(".admin-contact-row").last();
        await row.getByRole("combobox").selectOption("url");
        await row.getByLabel("Label", { exact: true }).fill("Test website");
        await row.getByLabel("Contact detail", { exact: true }).fill("https://example.com/validation");
        await page.locator("[data-editor] .ql-editor").fill("Test contact page text.");
        await page.getByRole("button", { name: "Save contacts", exact: true }).click();
        await status("Contacts saved");
        await page.reload();
        await page.locator("[data-editor] .ql-editor").waitFor();
        assert.equal(await page.locator(".admin-contact-row").last().getByLabel("Contact detail", { exact: true }).inputValue(), "https://example.com/validation");
        const contactPreview = await page.context().newPage();
        try {
            await contactPreview.goto(`${state.urls.portfolio}/contact`);
            const email = contactPreview.getByRole("link", { name: /Test email/ });
            await email.waitFor();
            assert.equal(await email.getAttribute("href"), "mailto:validation%40example.com");
            assert.equal(await contactPreview.getByRole("link", { name: /Test website/ }).getAttribute("href"), "https://example.com/validation");
        } finally { await contactPreview.close(); }
        await page.locator(".admin-contact-row").last().getByRole("button", { name: /Remove contact/ }).click();
        await page.getByRole("button", { name: "Save contacts", exact: true }).click();
        await status("Contacts saved");
        const savedContact = await (await api("get", contactPath)).json();
        assert.equal(savedContact.description.contacts.some(entry => entry.value === "https://example.com/validation"), false);
        console.log("Admin workflow: contact creation, public links, persistence and removal passed");

        await page.setViewportSize({ width: 390, height: 844 });
        await page.getByRole("link", { name: /Dashboard/ }).click();
        await page.getByRole("heading", { name: "Your creative workspace" }).waitFor();
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), "Mobile layout overflows");
        await page.getByRole("link", { name: /Contacts/ }).first().click();
        await page.locator("[data-editor] .ql-editor").waitFor();
        await page.screenshot({ path: path.join(stateDir, "admin-mobile-preview.png"), fullPage: true });
        console.log("PASS: admin create/cancel/validation/error recovery, edit/publish/hide/delete, media upload/cover/delete, profile/photo, contact CRUD/public links, mobile navigation");
    } finally {
        const cleanup = [];
        for (const id of projects) cleanup.push(api("delete", `${projectPath}/${id}`));
        cleanup.push(api("put", `${aboutPath}/${about.id}`, { data: { firstName: about.firstName, lastName: about.lastName, description: about.description } }));
        cleanup.push(api("put", `${contactPath}/${contact.id}`, { data: { description: contact.description } }));
        if (photoChanged) {
            if (originalPhoto) {
                const extension = originalPhoto.mimeType.split("/")[1]?.split(";")[0] || "png";
                cleanup.push(api("post", `${aboutPath}/${about.id}/image`, { multipart: { file: { name: `restored-profile.${extension}`, ...originalPhoto } } }));
            } else cleanup.push(api("delete", `${aboutPath}/${about.id}`));
        }
        const results = await Promise.allSettled(cleanup);
        assert.ok(results.every(result => result.status === "fulfilled"), "Failed to restore some test data; inspect the local test log");
        await page.setViewportSize({ width: 1280, height: 900 });
    }
}
