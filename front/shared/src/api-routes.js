/**
 * Centralized API route definitions.
 * All API paths live here so they're easy to find and update.
 */
export const API_ROUTES = Object.freeze({
    // About / Me
    ABOUT_ME: "/api/about/v1/me",
    ABOUT_ME_BY_ID: (id) => `/api/about/v1/me/${id}`,
    ABOUT_ME_IMAGE: (id) => `/api/about/v1/me/${id}/image`,

    // Contact
    CONTACT_INFO: "/api/contact/v1/information",
    CONTACT_INFO_BY_ID: (id) => `/api/contact/v1/information/${id}`,

    // Portfolio / Projects
    PROJECTS_V1: "/api/portfolio/v1/projects",
    PROJECTS_V2: "/api/portfolio/v2/projects",
    PROJECT_BY_ID: (id) => `/api/portfolio/v1/projects/${id}`,
    PROJECT_CONTENTS: (id) => `/api/portfolio/v1/projects/${id}/contents`,
    PROJECT_CONTENT: (id, contentId) => `/api/portfolio/v1/projects/${id}/contents/${contentId}`,
    PROJECT_THUMBNAIL: (id, contentId) => `/api/portfolio/v1/projects/${id}/thumbnails/${contentId}`,
});
