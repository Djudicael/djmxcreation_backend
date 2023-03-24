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
    async doPostMultipart({ path, body, authToken }) {
        const headers = {
            // 'Accept': 'application/json, text/plain, */*',
            // 'Content-Type': 'multipart/form-data'
        };
        if (authToken) {

            headers['Authorization'] = 'bearer ' + authToken;
        }
        const response = await fetch(config.rest_url + path, {
            headers: headers,
            method: 'POST',
            body: body
        });
        // return response.json();
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