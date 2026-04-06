export class BaseHttp {
    constructor(config) {
        this.config = config;
    }

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

    createMultipartHeaders(authToken) {
        const headers = {};

        if (authToken) {
            headers.Authorization = `Bearer ${authToken}`;
        }

        return headers;
    }

    buildUrl(path) {
        return this.config.rest_url + path;
    }

    async doPost({ path, body, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "POST",
            body: JSON.stringify(body),
        });

        return response.json();
    }

    async doPatchMultipart({ path, body, authToken }) {
        if (!(body instanceof FormData)) {
            throw new Error("Invalid request body");
        }

        const response = await fetch(this.buildUrl(path), {
            headers: this.createMultipartHeaders(authToken),
            method: "PATCH",
            body,
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        return response.json();
    }

    async doPostMultipart({ path, body, authToken }) {
        if (!(body instanceof FormData)) {
            throw new Error("Invalid request body");
        }

        const response = await fetch(this.buildUrl(path), {
            headers: this.createMultipartHeaders(authToken),
            method: "POST",
            body,
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        return response;
    }

    async doPutJson({ path, body, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "PUT",
            body: JSON.stringify(body),
        });

        return response.json();
    }

    async doPutRaw({ path, body, authToken }) {
        return fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "PUT",
            body: JSON.stringify(body),
        });
    }

    async doDeleteJson({ path, authToken }) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "DELETE",
        });

        return response.json();
    }

    async doDeleteRaw({ path, authToken }) {
        return fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(authToken),
            method: "DELETE",
        });
    }

    async doGet(path) {
        const response = await fetch(this.buildUrl(path), {
            headers: this.createJsonHeaders(),
        });

        return response.json();
    }
}
