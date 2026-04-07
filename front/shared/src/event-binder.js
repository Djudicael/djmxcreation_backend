/**
 * Manages dynamic event listeners on DOM elements that may be re-rendered.
 * Keeps track of previous handlers so they can be properly cleaned up
 * before rebinding, preventing memory leaks and duplicate handlers.
 *
 * Usage:
 *   this._imageBinder = new EventBinder();
 *   // After each render:
 *   this._imageBinder.bindAll(this.querySelectorAll('.remove-image'), 'click', (e) => this.deleteImage(e));
 *   // On disconnectedCallback:
 *   this._imageBinder.unbindAll();
 */
export class EventBinder {
    constructor() {
        this._entries = [];
    }

    /**
     * Unbind all previous listeners, then bind a new handler to each element.
     * @param {NodeList|Element[]} elements
     * @param {string} event
     * @param {EventListener} handler  — a single handler factory; each element gets its own wrapper.
     */
    bindAll(elements, event, handler) {
        this.unbindAll();
        for (const el of elements) {
            const wrapper = (e) => handler(e);
            el.addEventListener(event, wrapper);
            this._entries.push({ el, event, wrapper });
        }
    }

    unbindAll() {
        for (const { el, event, wrapper } of this._entries) {
            el.removeEventListener(event, wrapper);
        }
        this._entries = [];
    }
}
