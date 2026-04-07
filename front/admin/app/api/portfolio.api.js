import http from "../http/http.js";
import { AdminPortfolioApi } from "../../../shared/src/portfolio-api.js";

/** Singleton admin API — shares the singleton Http instance. */
const instance = new AdminPortfolioApi(http);
export default instance;
