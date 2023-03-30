import Http from '../http/http.js'
import Authentication from "../auth/authentication.js";
export default class PortfolioApi {
    constructor() {
        this.instance = new Http();
    }

    async createProject({ title, subTitle, client }) {
        // const auth = new Authentication();
        return await this.instance.doPost({ path: "/api/portfolio/v1/projects", body: { title, subTitle, client } });
    }

    async updateProjectMetadata(id, { title, subTitle, client }) {
        const auth = new Authentication();
        return await this.instance.doPut({ path: `/v1/portfolio/projects/${id}/metadata`, body: { title, subTitle, client }, authToken: auth.auth });
    }

    async updateDescription(id, { description }) {
        const auth = new Authentication();
        return await this.instance.doPut({ path: `/v1/portfolio/projects/${id}/description`, body: { description }, authToken: auth.auth });
    }

    async updateVisibility(id, { isVisible }) {
        const auth = new Authentication();
        return await this.instance.doPut({ path: `/v1/portfolio/projects/${id}/visibility`, body: { visible: isVisible }, authToken: auth.auth });
    }

    async deleteContent(id, contentID) {
        const auth = new Authentication();
        return await this.instance.doDelete({ path: `/v1/portfolio/projects/${id}/contents/${contentID}`, authToken: auth.auth });
    }

    async deleteProject(id) {
        const auth = new Authentication();
        return await this.instance.doDelete({ path: `/api/portfolio/v1/projects/${id}`});
    }

    async addContentToProject(id, { file }) {
        const auth = new Authentication();
        const data = new FormData()
        data.append('file', file);

        return await this.instance.doPostMultipart({ path: `/v1/portfolio/projects/${id}/contents`, body: data, authToken: auth.auth });
    }

    async getProjects() {
        return await this.instance.doGet("/api/portfolio/v1/projects");
    }
    async getShowReel() {
        return await this.instance.doGet("/portfolio/showreel");
    }

    async getAboutMe() {
        return await this.instance.doGet("/v1/about/me");
    }
    async updateAboutMeDescription(id, { description }) {
        const auth = new Authentication();
        await this.instance.doPut({ path: `/v1/about/me/${id}/description`, body: { description }, authToken: auth.auth });
    }
    async updateAboutMePicture(id, { file }) {

        const auth = new Authentication();
        const data = new FormData()
        data.append('file', file);

        return await this.instance.doPostMultipart({ path: `/v1/about/me/${id}/image`, body: data, authToken: auth.auth });
    }

    async deleteProfileImage(id) {
        const auth = new Authentication();
        return await this.instance.doDelete({ path: `/v1/about/me/${id}/image`, authToken: auth.auth });
    }

    async getContacts() {
        return await this.instance.doGet("/portfolio/contacts");
    }
    async getProject(projectId) {
        return await this.instance.doGet(`/api/portfolio/v1/projects/${projectId}`);
    }

}
