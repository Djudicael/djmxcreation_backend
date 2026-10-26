import assert from "node:assert/strict";
import path from "node:path";

// Deterministic public-page scenarios. Network fixtures stay in this browser
// context; the admin suite separately verifies the real API and persistence.
export async function runPortfolioWorkflows({ browser, state }) {
    const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
    context.setDefaultTimeout(15000);
    const page = await context.newPage();
    const failures = [];
    page.on("pageerror", error => failures.push(error.message));
    const root = state.urls.portfolio;
    const stateDir = path.dirname(process.env.DJMX_STACK_STATE);
    const projects = Array.from({ length: 14 }, (_, index) => ({
        id: String(index + 1), adult: index === 0, visible: true,
        metadata: { title: index === 0 ? "Adult study" : `Visual study ${index}`, subTitle: "A study of form and perspective", client: "Portfolio test" },
        thumbnail: { url: `${root}/portfolio-test/image-${index}.svg` },
        description: { ops: [{ insert: "An exploration of light, space and the shapes in between.\n" }] },
        contents: [0, 1].map(number => ({ url: `${root}/portfolio-test/image-${index}-${number}.svg`, mimeType: "image/svg+xml" })),
    }));
    const gallerySizes = [[1600, 900], [800, 1000], [1000, 1000], [1600, 1000], [700, 1000], [800, 1100]];
    projects[1].contents = gallerySizes.map((_, index) => ({ url: `${root}/portfolio-test/gallery-${index}.svg`, mimeType: "image/svg+xml" }));
    projects[1].contents.splice(5, 0, { url: "https://example.com/project.pdf", mimeType: "application/pdf" });
    projects[2].contents = [projects[1].contents[1]];
    projects[3].contents = [];
    let empty = false, unavailable = false, failNextPage = false;
    await context.route("**/portfolio-test/*.svg", route => {
        const match = route.request().url().match(/gallery-(\d+)\.svg/);
        if (match) {
            const index = Number(match[1]);
            const [width, height] = gallerySizes[index];
            return route.fulfill({ contentType: "image/svg+xml", body: `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 1000 1000" preserveAspectRatio="none"><rect width="1000" height="1000" fill="${index % 2 ? "#d4c9b5" : "#bac8b5"}"/><path d="M0 800L1000 100V1000H0Z" fill="#e9e5d3"/><circle cx="500" cy="400" r="230" fill="${index % 2 ? "#86745c" : "#465f49"}"/><rect x="24" y="24" width="952" height="952" fill="none" stroke="#202822" stroke-width="4"/></svg>` });
        }
        return route.fulfill({
        contentType: "image/svg+xml",
        body: '<svg xmlns="http://www.w3.org/2000/svg" width="720" height="900"><rect width="720" height="900" fill="#bcc7b5"/><circle cx="380" cy="430" r="240" fill="#718969"/><path d="M0 850L720 100V900H0Z" fill="#e9e5d3"/><circle cx="220" cy="230" r="100" fill="#465f49"/></svg>',
        });
    });
    await context.route("**/api/portfolio/**", route => {
        const url = new URL(route.request().url());
        if (unavailable) return route.fulfill({ status: 503, json: {} });
        if (url.pathname.includes("/v2/")) {
            const pageNumber = Number(url.searchParams.get("page"));
            if (failNextPage && pageNumber === 2) {
                failNextPage = false;
                return route.fulfill({ status: 503, json: {} });
            }
            const size = Number(url.searchParams.get("size"));
            return route.fulfill({ json: { projects: empty ? [] : projects.slice((pageNumber - 1) * size, pageNumber * size), totalPages: empty ? 0 : Math.ceil(projects.length / size), page: pageNumber, size } });
        }
        return route.fulfill({ json: projects.find(project => project.id === url.pathname.split("/").at(-1)) });
    });
    await context.route("**/api/about/v1/me", route => route.fulfill({ json: { firstName: "Creative", lastName: "Perspective", photoUrl: `${root}/portfolio-test/portrait.svg`, description: { ops: [{ insert: "A personal perspective on making and sharing work.\nAn editorial detail" }, { insert: "\n", attributes: { list: "bullet" } }] } } }));
    await context.route("**/api/contact/v1/information", route => route.fulfill({ json: { description: { ops: [{ insert: "For project enquiries and creative conversations.\n" }], contacts: [{ type: "email", label: "Email", value: "hello@example.com" }, { type: "url", label: "Elsewhere", value: "https://example.com" }] } } }));
    const fits = async () => assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), "Public page has horizontal overflow");
    try {
        await page.goto(root);
        await page.locator(".featured-image img").waitFor();
        assert.equal(await page.locator(".home-page a[href='/works/1']").count(), 0, "Adult work leaked onto home");
        await page.locator(".home-intro").getByRole("link", { name: "View works", exact: true }).waitFor();
        await page.screenshot({ path: path.join(stateDir, "portfolio-home-desktop.png"), fullPage: true });
        const menu = page.getByRole("button", { name: "Menu", exact: true });
        await menu.click();
        assert.equal(await page.locator("main").evaluate(element => element.inert), true);
        await page.keyboard.press("Shift+Tab");
        assert.equal(await page.locator(".menu-tog").evaluate(element => element === document.activeElement), true);
        await page.keyboard.press("Shift+Tab");
        assert.equal(await page.locator(".menu a").last().evaluate(element => element === document.activeElement), true);
        await page.keyboard.press("Escape");
        assert.equal(await menu.getAttribute("aria-expanded"), "false");
        assert.equal(await menu.evaluate(element => element === document.activeElement), true);
        await menu.click();
        await page.locator(".menu").evaluate(element => Promise.all(element.getAnimations({ subtree: true }).map(animation => animation.finished)));
        assert.equal(await page.locator(".menu a").first().evaluate(element => getComputedStyle(element).textDecorationLine), "underline");
        await page.screenshot({ path: path.join(stateDir, "portfolio-menu-desktop.png") });
        await page.getByRole("navigation").getByRole("link", { name: "Works/" }).click();
        await page.locator(".project").first().waitFor();
        console.log("Portfolio workflow: home and keyboard menu passed");
        assert.equal(await page.locator("main").evaluate(element => element.inert), false);
        await page.locator(".project").first().hover();
        await page.waitForFunction(() => document.querySelector("p-works")?.percent >= 1);
        assert.equal(await page.locator("canvas").getAttribute("data-censored"), "true");
        assert.equal(await page.locator("canvas").evaluate(element => getComputedStyle(element).filter), "blur(16px)");
        await page.screenshot({ path: path.join(stateDir, "portfolio-works-desktop.png") });
        await page.locator(".project").nth(1).hover();
        await page.waitForFunction(() => document.querySelector("canvas")?.dataset.censored === "false");
        await page.emulateMedia({ reducedMotion: "reduce" });
        await page.mouse.move(900, 500);
        await page.waitForFunction(() => {
            const works = document.querySelector("p-works");
            return works.currentX === works.targetX && works.currentY === works.targetY;
        });
        await page.emulateMedia({ reducedMotion: "no-preference" });
        console.log("Portfolio workflow: censored hover and reduced motion passed");
        failNextPage = true;
        await page.getByRole("button", { name: "Load more projects" }).click();
        await page.getByRole("status").filter({ hasText: "More works could not be loaded" }).waitFor();
        assert.equal(await page.locator(".project").count(), 6, "A failed page must preserve the list and allow retry");
        await page.getByRole("button", { name: "Load more projects" }).click();
        await page.waitForFunction(() => document.querySelectorAll(".project").length === 12);
        await page.getByRole("button", { name: "Load more projects" }).click();
        await page.waitForFunction(() => document.querySelectorAll(".project").length === 14);
        assert.equal(await page.getByRole("button", { name: "Load more projects" }).count(), 0);
        console.log("Portfolio workflow: pagination and retry passed");
        await page.locator(".project a").first().click();
        await page.getByRole("heading", { name: "Adult study", exact: true }).waitFor();
        assert.equal(await page.locator(".pro-image").count(), 0, "Adult project displayed before consent");
        await page.getByRole("button", { name: "View project", exact: true }).click();
        await page.locator(".gallery-image").first().click();
        await page.getByRole("dialog").waitFor();
        await page.keyboard.press("ArrowRight");
        assert.equal(await page.locator(".lightbox-count").innerText(), "2 / 2");
        await page.keyboard.press("Escape");
        assert.equal(await page.locator(".gallery-image").first().evaluate(element => element === document.activeElement), true);
        await page.getByRole("button", { name: "Hide adult content" }).click();
        assert.equal(await page.locator(".pro-image").count(), 0);
        await page.reload();
        await page.getByRole("button", { name: "View project", exact: true }).waitFor();
        await page.goto(`${root}/works/2`);
        console.log("Portfolio workflow: adult gate and keyboard gallery passed");
        await page.locator(".pro-image").first().waitFor();
        for (const image of await page.locator(".pro-image").all()) {
            await image.scrollIntoViewIfNeeded();
            await image.evaluate(image => image.decode());
        }
        const geometry = () => page.locator(".project-gallery").evaluate(gallery => {
            const box = element => {
                const { x, y, width, height } = element.getBoundingClientRect();
                return { x, y, width, height };
            };
            return {
                gallery: box(gallery),
                photos: [...gallery.querySelectorAll(".pro-image")].map(image => ({ ...box(image), ratio: image.naturalWidth / image.naturalHeight })),
            };
        });
        const desktop = await geometry();
        assert.equal(desktop.photos.length, 6);
        for (const image of desktop.photos) assert.ok(Math.abs(image.width / image.height - image.ratio) < .01, "An image was cropped or stretched");
        assert.ok(Math.abs(desktop.photos[1].y - desktop.photos[2].y) < 1, "Adjacent portraits should share a row");
        assert.ok(desktop.photos[2].x > desktop.photos[1].x + desktop.photos[1].width, "The pair needs a gutter");
        for (const index of [0, 3]) assert.ok(Math.abs(desktop.photos[index].width - desktop.gallery.width) < 1, "Wide images should span the gallery");
        for (const index of [4, 5]) {
            const photo = desktop.photos[index];
            assert.ok(Math.abs(photo.x + photo.width / 2 - desktop.gallery.x - desktop.gallery.width / 2) < 1, "An unpaired portrait should be centered");
        }
        await page.locator(".gallery-image").nth(2).click();
        assert.equal(await page.locator(".lightbox-count").innerText(), "3 / 6");
        await page.keyboard.press("ArrowRight");
        assert.equal(await page.locator(".lightbox-count").innerText(), "4 / 6", "Viewer order must match the gallery");
        await page.keyboard.press("Escape");
        assert.equal(await page.locator(".gallery-image").nth(2).evaluate(element => element === document.activeElement), true);
        await page.evaluate(() => scrollTo(0, 0));
        await page.screenshot({ path: path.join(stateDir, "portfolio-project-desktop.png"), fullPage: true });
        await page.setViewportSize({ width: 390, height: 844 });
        const mobile = await geometry();
        for (let index = 0; index < mobile.photos.length; index++) {
            assert.ok(Math.abs(mobile.photos[index].width - mobile.gallery.width) < 1, "Mobile images should fill one column");
            if (index) assert.ok(mobile.photos[index].y >= mobile.photos[index - 1].y + mobile.photos[index - 1].height, "Mobile images must follow reading order");
        }
        await fits();
        await page.screenshot({ path: path.join(stateDir, "portfolio-project-mobile.png"), fullPage: true });
        await page.setViewportSize({ width: 1440, height: 1000 });
        await page.goto(`${root}/works/3`);
        await page.locator(".pro-image").evaluate(image => image.decode());
        assert.equal(await page.locator(".gallery-image").count(), 1);
        await page.locator(".gallery-image").click();
        assert.equal(await page.locator(".lightbox-navigation").isVisible(), false);
        await page.keyboard.press("Escape");
        await page.goto(`${root}/works/4`);
        await page.locator(".project-heading h1").waitFor();
        assert.equal(await page.locator(".gallery-image").count(), 0);
        console.log("Portfolio workflow: mixed image ratios, pairs, unpaired portraits, files, single/empty galleries, viewer order and mobile layout passed");
        for (const route of ["/about", "/contact"]) {
            await page.goto(`${root}${route}`);
            await page.locator(".page-shell h1").waitFor();
            if (route === "/about") await page.getByText("An editorial detail", { exact: true }).waitFor();
            await fits();
            await page.screenshot({ path: path.join(stateDir, `portfolio-${route.slice(1)}-desktop.png`), fullPage: true });
        }
        assert.equal(await page.getByRole("link", { name: /Email hello/ }).getAttribute("href"), "mailto:hello%40example.com");
        for (const width of [390, 320, 768]) {
            await page.setViewportSize({ width, height: 844 });
            for (const route of ["/", "/works", "/works/1", "/works/2", "/about", "/contact"]) {
                await page.goto(`${root}${route}`);
                await page.locator("main h1").waitFor();
                await fits();
            }
        }
        await page.setViewportSize({ width: 390, height: 844 });
        await page.goto(root);
        await page.locator(".featured-image img").waitFor();
        await page.screenshot({ path: path.join(stateDir, "portfolio-home-mobile.png"), fullPage: true });
        await menu.click();
        await page.getByRole("navigation").getByRole("link", { name: "Contact/" }).click();
        await page.getByRole("heading", { name: "Contact.", exact: true }).waitFor();
        await page.screenshot({ path: path.join(stateDir, "portfolio-contact-mobile.png"), fullPage: true });
        empty = true;
        await page.goto(root);
        await page.locator(".visual-study").waitFor();
        await page.screenshot({ path: path.join(stateDir, "portfolio-home-empty.png"), fullPage: true });
        unavailable = true;
        await page.reload();
        await page.getByRole("button", { name: "Try again" }).waitFor();
        await page.locator(".home-intro").getByRole("link", { name: "View works", exact: true }).waitFor();
        unavailable = false;
        await page.getByRole("button", { name: "Try again" }).click();
        await page.waitForFunction(() => !document.querySelector(".quiet-status"));
        const touchContext = await browser.newContext({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });
        touchContext.setDefaultTimeout(15000);
        try {
            await touchContext.route("**/api/portfolio/**", route => route.fulfill({ json: route.request().url().includes("/v2/") ? { projects: projects.slice(0, 6), page: 1, size: 6, totalPages: 3 } : projects[0] }));
            const touchPage = await touchContext.newPage();
            await touchPage.goto(`${root}/works`);
            await touchPage.locator(".project a").first().waitFor();
            assert.equal(await touchPage.locator(".project-image").count(), 0, "Touch devices should not load hover-only previews");
            assert.equal(await touchPage.locator("canvas").isVisible(), false);
            await touchPage.locator(".project a").first().tap();
            await touchPage.getByRole("button", { name: "View project", exact: true }).waitFor();
            await touchPage.getByRole("button", { name: "Menu", exact: true }).tap();
            await touchPage.getByRole("navigation").getByRole("link", { name: "Home/" }).tap();
            await touchPage.getByRole("heading", { name: /DJMX/ }).waitFor();
        } finally { await touchContext.close(); }
        assert.deepEqual(failures, []);
        console.log("PASS: portfolio home/empty/retry, menu keyboard/mobile, NSFW hover/gate, pagination, gallery keyboard/focus, project/About/Contact, 320/390/768px layouts");
    } catch (error) {
        console.error("Portfolio scenario stopped at", new URL(page.url()).pathname);
        console.error((await page.locator("body").innerText()).slice(0, 1500));
        await page.screenshot({ path: path.join(stateDir, "portfolio-failure.png"), fullPage: true }).catch(() => {});
        throw error;
    } finally { await context.close(); }
}
