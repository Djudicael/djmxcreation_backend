import { Core } from "./core/core.js";
import { initKeycloak } from "./auth/security.js";

class App {
  constructor() {
    new Core();
  }
}

const onAuthenticatedCallback = async () => {
  if (window.location.pathname.startsWith("/preview/")) {
    const match = window.location.pathname.match(/^\/preview\/([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\/?$/i);
    if (!match) { document.body.textContent = "Invalid project preview address."; return; }
    try {
      const { showProjectPreview } = await import("./pages/project-preview.js");
      await showProjectPreview(match[1]);
    } catch { document.body.textContent = "Unable to open the preview. Please reload the page."; }
  } else new App();
};

initKeycloak(onAuthenticatedCallback);
