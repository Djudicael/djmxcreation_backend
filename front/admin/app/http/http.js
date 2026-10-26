import { config } from "../config/api.config.js";
import { BaseHttp } from "../../../shared/src/http-base.js";
import { getAccessToken } from "../auth/security.js";

class Http extends BaseHttp {
    constructor() {
        super(config, { tokenProvider: getAccessToken });
    }

    async doPut({ path, body, authToken }) {
        return this.doPutJson({ path, body, authToken });
    }

    async doPutVoid({ path, body, authToken }) {
        return this.doPutRaw({ path, body, authToken });
    }

    async doDelete({ path, authToken }) {
        return this.doDeleteRaw({ path, authToken });
    }

    async fetch(path, options) {
        const response = await super.fetch(path, options);
        if (response.ok && !["GET", "HEAD"].includes(options.method)) {
            // Media and thumbnail writes also change their parent project/list.
            this._cache.clear();
        }
        return response;
    }
}

/** Shared singleton — avoids creating a new instance per component. */
const httpInstance = new Http();
export default httpInstance;
