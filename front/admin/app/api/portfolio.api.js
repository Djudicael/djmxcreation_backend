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

    async deleteContent(id, contentID) {

        return await this.instance.doDelete({ path: `/api/portfolio/v1/projects/${id}/contents/${contentID}` });
    }

    async deleteProject(id) {
        const auth = new Authentication();
        return await this.instance.doDelete({ path: `/api/portfolio/v1/projects/${id}` });
    }

    async addContentToProject(id, { file }) {
        const data = new FormData()
        data.append('photo', file);
        return await this.instance.doPatchMultipart({ path: `/api/portfolio/v1/projects/${id}/contents`, body: data });
    }

    async getProjects() {
        return await this.instance.doGet("/api/portfolio/v1/projects");
    }

    async getAboutMe() {
        return await this.instance.doGet("/api/about/v1/me");
    }
    async updateAboutMeDescription(id, { firstName, lastName, description }) {

        await this.instance.doPut({ path: `/api/about/v1/me/${id}`, body: { firstName, lastName, description } });
    }
    async updateAboutMePicture(id, { file }) {

        const data = new FormData()
        data.append('file', file);

        return await this.instance.doPostMultipart({ path: `/api/about/v1/me/${id}/image`, body: data });
    }

    async deleteProfileImage(id) {
        return await this.instance.doDelete({ path: `/api/about/v1/me/${id}` });
    }

    async getContacts() {
        return await this.instance.doGet("/api/contact/v1/information");
    }

    async updateContactDescription(id, { description }) {

        await this.instance.doPut({ path: `/api/contact/v1/information/${id}`, body: { description } });
    }
    async getProject(projectId) {
        return await this.instance.doGet(`/api/portfolio/v1/projects/${projectId}`);
    }

    async updateProject(projectId, project) {
        console.log(project);

        return await this.instance.doPut({ path: `/api/portfolio/v1/projects/${projectId}`, body: project });
    }

}
