import { API_ROUTES } from "./api-routes.js";

/**
 * Base API class with shared read endpoints.
 */
export class PortfolioApiBase {
    /** @param {import('./http-base.js').BaseHttp} httpInstance */
    constructor(httpInstance) {
        this.instance = httpInstance;
    }

    /** @returns {Promise<object>} */
    async getAboutMe() {
        return this.instance.doGet(API_ROUTES.ABOUT_ME);
    }

    /** @returns {Promise<object>} */
    async getContacts() {
        return this.instance.doGet(API_ROUTES.CONTACT_INFO);
    }

    /**
     * @param {string} projectId
     * @returns {Promise<object>}
     */
    async getProject(projectId) {
        return this.instance.doGet(API_ROUTES.PROJECT_BY_ID(projectId));
    }
}

/**
 * Admin API — full CRUD operations.
 */
export class AdminPortfolioApi extends PortfolioApiBase {
    /**
     * @param {object} metadata
     * @param {string} metadata.title
     * @param {string} [metadata.subTitle]
     * @param {string} [metadata.client]
     * @returns {Promise<{id: string}>}
     */
    async createProject({ title, subTitle, client }) {
        return this.instance.doPost({
            path: API_ROUTES.PROJECTS_V1,
            body: { title, subTitle, client },
        });
    }

    /**
     * @param {string} id - Project ID.
     * @param {string} contentID - Content ID.
     * @returns {Promise<*>}
     */
    async deleteContent(id, contentID) {
        return this.instance.doDelete({
            path: API_ROUTES.PROJECT_CONTENT(id, contentID),
        });
    }

    /**
     * @param {string} id - Project ID.
     * @param {string} contentID - Content ID to set as thumbnail.
     * @returns {Promise<{url: string}>}
     */
    async addThumbnail(id, contentID) {
        return this.instance.doPut({
            path: API_ROUTES.PROJECT_THUMBNAIL(id, contentID),
        });
    }

    /**
     * @param {string} id - Project ID.
     * @returns {Promise<*>}
     */
    async deleteProject(id) {
        return this.instance.doDelete({
            path: API_ROUTES.PROJECT_BY_ID(id),
        });
    }

    /**
     * @param {string} id - Project ID.
     * @param {object} params
     * @param {File} params.file
     * @returns {Promise<*>}
     */
    async addContentToProject(id, { file }) {
        const data = new FormData();
        data.append('photo', file);
        return this.instance.doPatchMultipart({
            path: API_ROUTES.PROJECT_CONTENTS(id),
            body: data,
        });
    }

    /** @returns {Promise<object[]>} */
    async getProjects() {
        return this.instance.doGet(API_ROUTES.PROJECTS_V1);
    }

    /**
     * @param {string} id - About-me record ID.
     * @param {object} data
     * @param {string} data.firstName
     * @param {string} data.lastName
     * @param {*} data.description
     * @returns {Promise<*>}
     */
    async updateAboutMeDescription(id, { firstName, lastName, description }) {
        return this.instance.doPut({
            path: API_ROUTES.ABOUT_ME_BY_ID(id),
            body: { firstName, lastName, description },
        });
    }

    /**
     * @param {string} id
     * @param {object} params
     * @param {File} params.file
     * @returns {Promise<*>}
     */
    async updateAboutMePicture(id, { file }) {
        const data = new FormData();
        data.append('file', file);
        return this.instance.doPostMultipart({
            path: API_ROUTES.ABOUT_ME_IMAGE(id),
            body: data,
        });
    }

    /**
     * @param {string} id
     * @returns {Promise<*>}
     */
    async deleteProfileImage(id) {
        return this.instance.doDelete({ path: API_ROUTES.ABOUT_ME_BY_ID(id) });
    }

    /**
     * @param {string} id
     * @param {object} data
     * @param {*} data.description
     * @returns {Promise<*>}
     */
    async updateContactDescription(id, { description }) {
        return this.instance.doPut({
            path: API_ROUTES.CONTACT_INFO_BY_ID(id),
            body: { description },
        });
    }

    /**
     * @param {string} projectId
     * @param {object} project
     * @returns {Promise<Response>}
     */
    async updateProject(projectId, project) {
        return this.instance.doPutVoid({
            path: API_ROUTES.PROJECT_BY_ID(projectId),
            body: project,
        });
    }
}

/**
 * Public API — read-only with pagination.
 */
export class PublicPortfolioApi extends PortfolioApiBase {
    /**
     * @param {object} params
     * @param {number} params.page
     * @param {number} params.pageSize
     * @returns {Promise<{totalPages: number, page: number, size: number, projects: object[]}>}
     */
    async getProjects({ page, pageSize }) {
        return this.instance.doGet(
            `${API_ROUTES.PROJECTS_V2}?page=${page}&size=${pageSize}&visible=true`
        );
    }
}
