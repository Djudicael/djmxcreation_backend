import { buildApp } from "../build/build-app.js";

buildApp().catch((error) => {
    console.error(error);
    process.exit(1);
});


