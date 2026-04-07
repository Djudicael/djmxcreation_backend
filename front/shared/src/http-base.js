/**
 * Lightweight TTL cache for GET responses.
 * @template T
 */
class ResponseCache {
    /** @param {number} ttlMs - Time-to-live in milliseconds (default 30 s). */
    constructor(ttlMs = 30_000) {
        /** @type {Map<string, {data: T, expiresAt: number}>} */
        this._entries = new Map();
        this._ttlMs = ttlMs;
    }

    /**
     * @param {string} key
     * @returns {T | undefined}
     */
    get(key) {
        const entry = this._entries.get(key);
        if (!entry) {
            return undefined;
        }
        if (Date.now() > entry.expiresAt) {
            this._entries.delete(key);
            return undefined;
        }
        return entry.data;
    }

    /**
     * @param {string} key
     * @param {T} data
     */
    set(key, data) {
        this._entries.set(key, { data, expiresAt: Date.now() + this._ttlMs });
    }

    /** Invalidate a single key or all keys matching a prefix. */
    invalidate(keyOrPrefix) {
        if (this._entries.has(keyOrPrefix)) {
            this._entries.delete(keyOrPrefix);
            return;
        }
        for (const k of this._entries.keys()) {
            if (k.startsWith(keyOrPrefix)) {
                this._entries.delete(k);
            }
        }
    }

    clear() {
        this._entries.clear();
    }
}

/**
 * Base HTTP client with AbortController support, response validation,
 * and an optional GET cache.
 */
export class BaseHttp {
    /**
     * @param {object} config
     * @param {string} config.rest_url - Base URL for all requests.
     */
    constructor(config) {
        this.config = config;
        /** @type {AbortController | null} */
        this._controller = null;
        this._cache = new ResponseCache();
    }

    /** Create a fresh AbortController for this client's lifetime. */
    connect() {
        this.abort();
        this._controller = new AbortController();
    }

    /** Abort all in-flight requests made through this client. */
    abort() {
        this._controller?.abort();
        this._controller = null;
    }

    /** @returns {AbortSignal | undefined} */
    get _signal() {
        return this._controller?.signal;
    }

    /**
     * @param {string} [authToken]
     * @returns {HeadersInit}
     */
    createJsonHeaders(authToken) {
        const headers = {
            Accept: "application/json, text/plain, */*",
            "Content-Type": "application/json",
        };

        if (authToken) {
            headers.Authorization = `Bearer ${authToken}`;
        }

        return headers;
    }

    /**
     * @param {string} [authToken]
     * @returns {HeadersInit}
     */
    createMultipartHeaders(authToken) {
        const headers = {};

        if (authToken) {
            headers.Authorization = `Bearer ${authToken}`;
        }

        return headers;
    }

    /**
     * @param {string} path
     * @returns {string}
     */
    buildUrl(path) {
        return this.config.rest_url + path;
    }

    /**
     * @param {Response} response
     * @throws {Error} on non-2xx status.
     */
    assertOk(response) {
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {*} options.body
     * @param {string} [options.authToken]
     * @returns {Promise<*>}
     */
    async doPost({ path, body, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "POST",
            body: JSON.stringify(body),
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response.json();
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {FormData} options.body
     * @param {string} [options.authToken]
     * @returns {Promise<*>}
     */
    async doPatchMultipart({ path, body, authToken }) {
        if (!(body instanceof FormData)) {
            throw new Error("Invalid request body");
        }

        const response = await fetch(this.buildUrl(path), {
            headers: this.createMultipartHeaders(authToken),
            method: "PATCH",
            body,
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response.json();
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {FormData} options.body
     * @param {string} [options.authToken]
     * @returns {Promise<Response>}
     */
    async doPostMultipart({ path, body, authToken }) {
        if (!(body instanceof FormData)) {
            throw new Error("Invalid request body");
        }

        const response = await fetch(this.buildUrl(path), {
            headers: this.createMultipartHeaders(authToken),
            method: "POST",
            body,
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response;
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {*} options.body
     * @param {string} [options.authToken]
     * @returns {Promise<*>}
     */
    async doPutJson({ path, body, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "PUT",
            body: JSON.stringify(body),
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response.json();
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {*} options.body
     * @param {string} [options.authToken]
     * @returns {Promise<Response>}
     */
    async doPutRaw({ path, body, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "PUT",
            body: JSON.stringify(body),
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response;
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {string} [options.authToken]
     * @returns {Promise<*>}
     */
    async doDeleteJson({ path, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "DELETE",
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response.json();
    }

    /**
     * @param {object} options
     * @param {string} options.path
     * @param {string} [options.authToken]
     * @returns {Promise<Response>}
     */
    async doDeleteRaw({ path, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "DELETE",
            signal: this._signal,
        });

        this.assertOk(response);
        this._cache.invalidate(path.split("?")[0]);
        return response;
    }

    /**
     * GET with transparent TTL cache.
     * @param {string} path
     * @returns {Promise<*>}
     */
    async doGet(path) {
        const cached = this._cache.get(path);
        if (cached !== undefined) {
            return cached;
        }

        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(),
            signal: this._signal,
        });

        this.assertOk(response);
        const data = await response.json();
        this._cache.set(path, data);
        return data;
    }
}
