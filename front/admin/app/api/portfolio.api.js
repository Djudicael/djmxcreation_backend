import Http from "../http/http.js";
import { AdminPortfolioApi } from "../../../shared/src/portfolio-api.js";

export default class PortfolioApi extends AdminPortfolioApi {
  constructor() {
    super(new Http());
  }
}
