import { buildApp } from "../build/build-app.js";

buildApp({ extraStyles: { "preview.css": "style/preview.css" } }).catch((error) => {
    console.error(error);
    process.exit(1);
});


