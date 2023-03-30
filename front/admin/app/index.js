
import { Core } from "./core/core.js";

import { initKeycloak } from "./auth/security.js";

class App {
    constructor() {
        new Core();
    }
}

const onAuthenticatedCallback = () => {
    new App();
}
new App();
// initKeycloak(onAuthenticatedCallback);

