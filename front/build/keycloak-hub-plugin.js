import { readFile } from "node:fs/promises";

// Keycloak JS 26 assumes refresh tokens are JWTs. Hub refresh tokens are opaque.
// Preserve the original token for refresh/revocation; only optional parsed
// metadata is absent. Access/ID token parsing and nonce validation stay intact.
export const keycloakHubPlugin = {
    name: "keycloak-hub-opaque-refresh",
    setup(build) {
        build.onLoad({ filter: /[\\/]keycloak-js[\\/]lib[\\/]keycloak\.js$/ }, async ({ path }) => {
            const source = await readFile(path, "utf8");
            const original = "this.refreshTokenParsed = decodeToken(refreshToken)";
            if (source.split(original).length !== 2) {
                throw new Error("Keycloak JS changed: review the Hub opaque refresh-token compatibility patch.");
            }
            return {
                loader: "js",
                contents: source.replace(original,
                    "this.refreshTokenParsed = refreshToken.split('.').length === 3 ? decodeToken(refreshToken) : undefined"),
            };
        });
    },
};
