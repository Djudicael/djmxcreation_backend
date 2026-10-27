import { html, render } from "lit-html";
import WorkComponent from "../../../portfolio/app/pages/work.component.js";
import api from "../api/portfolio.api.js";

export async function showProjectPreview(id) {
    const editUrl = `/projects/${id}`;
    class PrivatePreview extends WorkComponent {
        constructor() { super(); this.projectId = id; }
        get api() { return api; }
        get worksUrl() { return editUrl; }
    }
    customElements.define("p-private-preview", PrivatePreview);
    const style = document.querySelector('link[rel="stylesheet"]');
    await new Promise((resolve, reject) => {
        style.onload = resolve;
        style.onerror = () => reject(new Error("Preview styles could not be loaded."));
        style.href = "/preview.css";
    });
    const robots = document.createElement("meta");
    robots.name = "robots"; robots.content = "noindex, nofollow";
    document.head.append(robots);
    document.body.replaceChildren();
    render(html`<header class="preview-banner"><div><strong>Private preview</strong><p>Saved changes only. This preview requires an administrator session.</p></div><nav aria-label="Preview actions"><button class="text-link" @click=${() => window.location.reload()}>Reload preview</button><a class="action-link" href=${editUrl}>Back to editor</a></nav></header><main><p-private-preview></p-private-preview></main>`, document.body);
}
