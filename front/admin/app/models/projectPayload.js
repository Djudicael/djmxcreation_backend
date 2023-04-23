export default class ProjectPayload {
    constructor({ metadata, description, visible, adult }) {
        this.metadata = metadata;
        this.description = description;
        this.visible = visible;
        this.adult = adult;
    }
}