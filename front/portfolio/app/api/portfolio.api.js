import http from '../http/http.js';
import { PublicPortfolioApi } from '../../../shared/src/portfolio-api.js';

/** Singleton public API — shares the singleton Http instance. */
const instance = new PublicPortfolioApi(http);
export default instance;
