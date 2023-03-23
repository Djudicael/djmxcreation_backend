
import Http from '../http/http.js'
export default class Authentication {

    get auth() {
        return JSON.parse(localStorage.getItem('auth'));
    }

    set auth(value) {
        localStorage.setItem('auth', JSON.stringify(value));
    }

    async doAuthentication({ username, password }) {
        const user = {
            username, password
        };
        const instance = new Http();
        const { access_token, refresh_token } = await instance.doPost({ path: '/v1/authentication/login', body: user });
        this.auth = access_token;
    }

}
