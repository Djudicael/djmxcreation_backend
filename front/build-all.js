import { spawn } from "child_process";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const npmCommand = process.platform === "win32" ? "npm.cmd" : "npm";

function runBuild(name, cwd) {
    return new Promise((resolve, reject) => {
        const child = spawn(npmCommand, ["run", "build"], {
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
            if (code === 0) {
                resolve();
            } else {
                reject(new Error(`${name} build failed with code ${code}`));
            }
        });
    });
}

await runBuild("admin", path.join(__dirname, "admin"));
await runBuild("portfolio", path.join(__dirname, "portfolio"));
