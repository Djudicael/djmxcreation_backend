// Exercise the actual keycloak-js -> Hub -> protected API browser round trip.
import fs from "node:fs";
import path from "node:path";
import assert from "node:assert/strict";
import { pathToFileURL } from "node:url";
import { runAdminWorkflows } from "./admin-workflows.mjs";
import { runPortfolioWorkflows } from "./portfolio-workflows.mjs";

const state = JSON.parse(fs.readFileSync(process.env.DJMX_STACK_STATE, "utf8"));
const credentials = JSON.parse(fs.readFileSync(process.env.DJMX_STACK_CREDENTIALS, "utf8"));
const playwrightPath = path.join(state.hub_dir, "front/admin/node_modules/playwright/index.mjs");
const { chromium } = await import(pathToFileURL(playwrightPath));
const browser = await chromium.launch({ headless: true });
let page;
const diagnostics = [];
try {
    const context = await browser.newContext();
    page = await context.newPage();
    page.on("console", message => {
        if (["error", "warning"].includes(message.type())) diagnostics.push(message.text());
    });
    const failures = [];
    page.on("pageerror", error => failures.push(error.message));
    await page.goto(state.urls.admin);
    await page.waitForURL(url => url.origin === state.urls.hub && url.pathname.includes("/realms/portfolio/"));
    await page.locator('input[name="email"], input[name="username"], #email').first().fill(credentials.email);
    await page.locator('input[type="password"]').fill(credentials.password);
    await page.locator('button[type="submit"]').click();
    // Fresh clients may require an initial consent grant.
    const result = await Promise.race([
        page.waitForURL(url => url.origin === state.urls.admin, { timeout: 30000 }).then(() => "admin"),
        page.locator('button:has-text("Allow"), button:has-text("Approve"), button:has-text("Accept")').first().waitFor({ timeout: 30000 }).then(() => "consent"),
    ]);
    if (result === "consent") {
        await page.locator('button:has-text("Allow"), button:has-text("Approve"), button:has-text("Accept")').first().click();
        await page.waitForURL(url => url.origin === state.urls.admin);
    }
    await page.locator("router-outlet").waitFor();
    await page.waitForFunction(() => document.querySelector("router-outlet")?.children.length > 0);
    // The app's authenticated HTTP client must send the Hub access token.
    const api = page.waitForResponse(response => response.url().includes("/api/portfolio/v1/projects") && response.status() === 200);
    await page.locator('a[href="/projects"]').first().click();
    const authenticatedResponse = await api;
    const authorization = await authenticatedResponse.request().headerValue("authorization");
    // Re-enter a deep URL, including application query/hash state, through SSO.
    const deepUrl = `${state.urls.admin}/projects?view=all#list`;
    await page.goto(deepUrl);
    await page.locator("router-outlet").waitFor();
    await page.waitForURL(deepUrl);
    const reloadedApi = page.waitForResponse(response => response.url().includes("/api/portfolio/v1/projects") && response.status() === 200);
    await page.reload();
    await reloadedApi;
    await page.waitForURL(deepUrl);
    await page.locator("router-outlet").waitFor();
    await runAdminWorkflows({ page, state, authorization });
    await runPortfolioWorkflows({ browser, state });
    await page.getByRole("button", { name: "Sign out", exact: true }).click();
    await page.waitForURL(url => url.origin === state.urls.hub && url.pathname.includes("/realms/portfolio/login"));
    await page.locator('input[type="password"]').waitFor();
    await page.goto(state.urls.portfolio);
    await page.locator("router-outlet").waitFor();
    assert.deepEqual(failures, []);
    console.log("PASS: browser PKCE sign-in, authenticated deep-link/reload with query/hash preserved, admin API access, and public portfolio");
} catch (error) {
    if (page) {
        const url = new URL(page.url());
        console.error(`Browser stopped at ${url.origin}${url.pathname}`);
        console.error((await page.locator("body").innerText()).slice(0, 2000));
        console.error(diagnostics.join("\n"));
    }
    throw error;
} finally {
    await browser.close();
}
