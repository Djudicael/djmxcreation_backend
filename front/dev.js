import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";

function runApp(name, cwd) {
    const child = spawn(npmCommand, ["run", "dev"], {
        cwd,
        stdio: ["inherit", "pipe", "pipe"],
        shell: false,
    });

    child.stdout.on("data", (chunk) => {
        process.stdout.write(`[${name}] ${chunk}`);
    });

    child.stderr.on("data", (chunk) => {
        process.stderr.write(`[${name}] ${chunk}`);
    });

    child.on("exit", (code) => {
        if (shuttingDown) {
            return;
        }

        console.error(`[${name}] exited with code ${code}`);
        shutdown(1);
    });

    return child;
}

let shuttingDown = false;
const children = [
    runApp("admin", path.join(__dirname, "admin")),
    runApp("portfolio", path.join(__dirname, "portfolio")),
];

function shutdown(code = 0) {
    if (shuttingDown) {
        return;
    }

    shuttingDown = true;
    for (const child of children) {
        if (!child.killed) {
            child.kill("SIGTERM");
        }
    }

    setTimeout(() => process.exit(code), 300);
}

process.on("SIGINT", () => shutdown(0));
process.on("SIGTERM", () => shutdown(0));
