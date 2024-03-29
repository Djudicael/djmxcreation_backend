import Http from '../http/http.js'

export default class PortfolioApi {
    constructor() {
        this.instance = new Http();
    }

    async getProjects({ page, pageSize }) {
        return await this.instance.doGet(`/api/portfolio/v2/projects?page=${page}&size=${pageSize}&visible=true`);
    }

    async getAboutMe() {
        return await this.instance.doGet("/api/about/v1/me");
    }

    async getContacts() {
        return await this.instance.doGet("/api/contact/v1/information");
    }

    async getProject(projectId) {
        return await this.instance.doGet(`/api/portfolio/v1/projects/${projectId}`);
    }
}
