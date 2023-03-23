import Authentication from "./authentication.js";
export default class AuthDefender {
    static canActivate() {
        const instance = new Authentication();
        return instance.auth ? true : false;
    }
}