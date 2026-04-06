import Http from '../http/http.js'
import { PublicPortfolioApi } from '../../../shared/src/portfolio-api.js';

export default class PortfolioApi extends PublicPortfolioApi {
    constructor() {
        super(new Http());
    }
}
