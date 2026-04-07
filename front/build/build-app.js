import { createRequire } from "module";
import * as fs from "fs";
import * as path from "path";

const require = createRequire(path.join(process.cwd(), "package.json"));
const esbuild = require("esbuild");

function parseEnvFile(filePath) {
    if (!fs.existsSync(filePath)) {
        return {};
    }

    const content = fs.readFileSync(filePath, "utf8");
    const values = {};

    for (const rawLine of content.split(/\r?\n/)) {
        const line = rawLine.trim();
        if (!line || line.startsWith("#")) {
            continue;
        }

        const separatorIndex = line.indexOf("=");
        if (separatorIndex <= 0) {
            continue;
        }

        const key = line.slice(0, separatorIndex).trim();
        const value = line.slice(separatorIndex + 1).trim().replace(/^['\"]|['\"]$/g, "");
        if (key) {
            values[key] = value;
        }
    }

    return values;
}

function loadBuildEnv(cwd) {
    const frontRoot = path.resolve(cwd, "..");
    const envSources = [
        path.join(frontRoot, ".env"),
        path.join(frontRoot, ".env.local"),
        path.join(cwd, ".env"),
        path.join(cwd, ".env.local"),
    ];

    const fileEnv = {};
    for (const envPath of envSources) {
        Object.assign(fileEnv, parseEnvFile(envPath));
    }

    return {
        ...fileEnv,
        ...process.env,
    };
}

function ensureDir(dirPath) {
    if (!fs.existsSync(dirPath)) {
        fs.mkdirSync(dirPath, { recursive: true });
    }
}

function copyIfExists(sourcePath, destinationPath) {
    if (!fs.existsSync(sourcePath)) {
        return;
    }

    const stat = fs.lstatSync(sourcePath);
    if (stat.isDirectory()) {
        fs.cpSync(sourcePath, destinationPath, { recursive: true });
        return;
    }

    ensureDir(path.dirname(destinationPath));
    fs.copyFileSync(sourcePath, destinationPath);
}

function getBuildOptions({
    entryPoint,
    cssEntryPoint,
    outDir,
    minify,
    sourcemap,
    target,
    define,
}) {
    return {
        js: {
            entryPoints: [entryPoint],
            bundle: true,
            minify,
            sourcemap,
            target,
            define,
            outfile: path.join(outDir, "js", "index.js"),
        },
        css: {
            entryPoints: [cssEntryPoint],
            bundle: true,
            minify,
            sourcemap,
            target,
            outfile: path.join(outDir, "style.css"),
        },
    };
}

export async function buildApp({
    entryPoint = "app/index.js",
    cssEntryPoint = "style/style.css",
    indexFile = "index.html",
    outDir = "dist",
    assetsFolder = "ressource",
    minify = true,
    sourcemap = process.env.NODE_ENV !== "production",
    target = ["chrome58"],
} = {}) {
    ensureDir(outDir);
    ensureDir(path.join(outDir, "js"));

    copyIfExists(indexFile, path.join(outDir, "index.html"));
    copyIfExists(assetsFolder, path.join(outDir, assetsFolder));

    const env = loadBuildEnv(process.cwd());
    const define = {
        "import.meta.env": JSON.stringify({
            BACKEND_API_URL: env.BACKEND_API_URL || "",
            KEYCLOAK_URL: env.KEYCLOAK_URL || "",
            KEYCLOAK_REALM: env.KEYCLOAK_REALM || "",
            KEYCLOAK_CLIENT_ID: env.KEYCLOAK_CLIENT_ID || "",
        }),
    };

    const buildOptions = getBuildOptions({
        entryPoint,
        cssEntryPoint,
        outDir,
        minify,
        sourcemap,
        target,
        define,
    });

    await esbuild.build(buildOptions.js);
    await esbuild.build(buildOptions.css);
}

export async function watchApp({
    entryPoint = "app/index.js",
    cssEntryPoint = "style/style.css",
    indexFile = "index.html",
    outDir = "dist",
    assetsFolder = "ressource",
    minify = false,
    sourcemap = true,
    target = ["chrome58"],
} = {}) {
    ensureDir(outDir);
    ensureDir(path.join(outDir, "js"));

    copyIfExists(indexFile, path.join(outDir, "index.html"));
    copyIfExists(assetsFolder, path.join(outDir, assetsFolder));

    const env = loadBuildEnv(process.cwd());
    const define = {
        "import.meta.env": JSON.stringify({
            BACKEND_API_URL: env.BACKEND_API_URL || "",
            KEYCLOAK_URL: env.KEYCLOAK_URL || "",
            KEYCLOAK_REALM: env.KEYCLOAK_REALM || "",
            KEYCLOAK_CLIENT_ID: env.KEYCLOAK_CLIENT_ID || "",
        }),
    };

    const buildOptions = getBuildOptions({
        entryPoint,
        cssEntryPoint,
        outDir,
        minify,
        sourcemap,
        target,
        define,
    });

    const jsContext = await esbuild.context(buildOptions.js);
    const cssContext = await esbuild.context(buildOptions.css);

    await jsContext.watch();
    await cssContext.watch();

    return {
        async dispose() {
            await jsContext.dispose();
            await cssContext.dispose();
        },
    };
}
