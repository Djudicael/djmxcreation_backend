export default class ProjectPayload {
    constructor({ metadata, description, visible }) {
        this.metadata = metadata;
        this.description = description;
        this.visible = visible;
    }
}