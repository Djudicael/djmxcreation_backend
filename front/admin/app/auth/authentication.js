import http from '../http/http.js'

const AUTH_KEY = 'auth';

export default class Authentication {

    get auth() {
        try {
            const raw = localStorage.getItem(AUTH_KEY);
            if (!raw) {
                return null;
            }
            return JSON.parse(raw);
        } catch {
            localStorage.removeItem(AUTH_KEY);
            return null;
        }
    }

    set auth(value) {
        if (value == null) {
            localStorage.removeItem(AUTH_KEY);
            return;
        }
        localStorage.setItem(AUTH_KEY, JSON.stringify(value));
    }

    async doAuthentication({ username, password }) {
        const { access_token } = await http.doPost({
            path: '/v1/authentication/login',
            body: { username, password },
        });
        this.auth = access_token;
    }
}
