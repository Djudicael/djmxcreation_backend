import { config } from "../config/api.config.js";
export default class Http {
    constructor() {
    }

    async doPost({ path, body, authToken }) {
        const headers = {
            'Accept': 'application/json, text/plain, */*',
            'Content-Type': 'application/json',
        };
        if (authToken) {

            headers['Authorization'] = 'bearer ' + authToken;
        }
        const response = await fetch(config.rest_url + path, {
            headers: headers,
            method: 'POST',
            body: JSON.stringify(body)
        });
        return response.json();
    }

    async doPatchMultipart({ path, body, authToken }) {
        const headers = {
            // 'Accept': 'application/json, text/plain, */*',
            // use 'multipart/form-data' instead of 'application/json'
            // 'Content-Type': 'multipart/form-data'
        };

        if (authToken) {
            headers['Authorization'] = 'Bearer ' + authToken;
        }

        try {
            // Ensure that body is a FormData instance
            if (!(body instanceof FormData)) {
                throw new Error('Invalid request body');
            }

            const response = await fetch(config.rest_url + path, {
                headers: headers,
                method: 'PATCH',
                body: body
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            return response.json();
        } catch (error) {
            console.error(error);
            // Handle error appropriately
            throw error; // re-throw error to propagate it up the call stack
        }
    }


    async doPostMultipart({ path, body, authToken }) {
        const headers = {
            // 'Accept': 'application/json, text/plain, */*',
            // 'Content-Type': 'multipart/form-data'
        };

        if (authToken) {
            headers['Authorization'] = 'bearer ' + authToken;
        }

        try {
            // Ensure that body is a FormData instance
            if (!(body instanceof FormData)) {
                throw new Error('Invalid request body');
            }

            const response = await fetch(config.rest_url + path, {
                headers: headers,
                method: 'POST',
                body: body
            });

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            return response;
        } catch (error) {
            console.error(error);
            // Handle error appropriately
        }
    }

    async doPut({ path, body, authToken }) {
        const headers = {
            'Accept': 'application/json, text/plain, */*',
            'Content-Type': 'application/json',
        };
        if (authToken) {

            headers['Authorization'] = 'bearer ' + authToken;
        }
        const response = await fetch(config.rest_url + path, {
            headers: headers,
            method: 'PUT',
            body: JSON.stringify(body)
        });

        return response.json();
    }

    async doDelete({ path, authToken }) {
        const headers = {
            'Accept': 'application/json, text/plain, */*',
            'Content-Type': 'application/json',
        };
        if (authToken) {

            headers['Authorization'] = 'bearer ' + authToken;
        }
        const response = await fetch(config.rest_url + path, {
            headers: headers,
            method: 'DELETE',
        });
        return response.json();
    }

    async doGet(path) {
        const headers = {
            'Accept': 'application/json, text/plain, */*',
            'Content-Type': 'application/json',
        };
        const response = await fetch(config.rest_url + path, {
            headers: headers
        });

        return response.json();
    }
}