export class PortfolioApiBase {
    constructor(httpInstance) {
        this.instance = httpInstance;
    }

    async getAboutMe() {
        return this.instance.doGet('/api/about/v1/me');
    }

    async getContacts() {
        return this.instance.doGet('/api/contact/v1/information');
    }

    async getProject(projectId) {
        return this.instance.doGet(`/api/portfolio/v1/projects/${projectId}`);
    }
}

export class AdminPortfolioApi extends PortfolioApiBase {
    async createProject({ title, subTitle, client }) {
        return this.instance.doPost({
            path: '/api/portfolio/v1/projects',
            body: { title, subTitle, client },
        });
    }

    async deleteContent(id, contentID) {
        return this.instance.doDelete({
            path: `/api/portfolio/v1/projects/${id}/contents/${contentID}`,
        });
    }

    async addThumbnail(id, contentID) {
        return this.instance.doPut({
            path: `/api/portfolio/v1/projects/${id}/thumbnails/${contentID}`,
        });
    }

    async deleteProject(id) {
        return this.instance.doDelete({
            path: `/api/portfolio/v1/projects/${id}`,
        });
    }

    async addContentToProject(id, { file }) {
        const data = new FormData();
        data.append('photo', file);
        return this.instance.doPatchMultipart({
            path: `/api/portfolio/v1/projects/${id}/contents`,
            body: data,
        });
    }

    async getProjects() {
        return this.instance.doGet('/api/portfolio/v1/projects');
    }

    async updateAboutMeDescription(id, { firstName, lastName, description }) {
        return this.instance.doPut({
            path: `/api/about/v1/me/${id}`,
            body: { firstName, lastName, description },
        });
    }

    async updateAboutMePicture(id, { file }) {
        const data = new FormData();
        data.append('file', file);

        return this.instance.doPostMultipart({
            path: `/api/about/v1/me/${id}/image`,
            body: data,
        });
    }

    async deleteProfileImage(id) {
        return this.instance.doDelete({ path: `/api/about/v1/me/${id}` });
    }

    async updateContactDescription(id, { description }) {
        return this.instance.doPut({
            path: `/api/contact/v1/information/${id}`,
            body: { description },
        });
    }

    async updateProject(projectId, project) {
        return this.instance.doPutVoid({
            path: `/api/portfolio/v1/projects/${projectId}`,
            body: project,
        });
    }
}

export class PublicPortfolioApi extends PortfolioApiBase {
    async getProjects({ page, pageSize }) {
        return this.instance.doGet(
            `/api/portfolio/v2/projects?page=${page}&size=${pageSize}&visible=true`
        );
    }
}
