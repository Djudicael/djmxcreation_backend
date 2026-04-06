import http from "http";
import app from "./app.js";
import { watchApp } from "../build/build-app.js";

const port = 3009;

const watcher = await watchApp({
    minify: false,
    sourcemap: true,
});

const server = http.createServer(app);

server.listen(port, () => {
    console.log(`Portfolio dev server listening on port ${port}`);
});

const shutdown = async () => {
    await watcher.dispose();
    server.close(() => process.exit(0));
};

process.on("SIGINT", shutdown);
process.on("SIGTERM", shutdown);
