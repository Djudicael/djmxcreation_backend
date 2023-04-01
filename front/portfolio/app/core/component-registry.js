export class ComponentRegistry {
    static register(components) {
        components.forEach(comp => {
            const {
                tagName,
                component
            } = comp;
            window.customElements.define(tagName, component);
        });
    }
}