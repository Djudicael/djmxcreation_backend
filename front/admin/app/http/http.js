import { config } from "../config/api.config.js";
import { BaseHttp } from "../../../shared/src/http-base.js";

export default class Http extends BaseHttp {
    constructor() {
        super(config);
    }

    async doPut({ path, body, authToken }) {
        return this.doPutJson({ path, body, authToken });
    }

    async doPutVoid({ path, body, authToken }) {
        return this.doPutRaw({ path, body, authToken });
    }

    async doDelete({ path, authToken }) {
        return this.doDeleteJson({ path, authToken });
    }
}