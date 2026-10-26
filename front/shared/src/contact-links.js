/** Render only supported, safe contact links, including for legacy stored data. */
export function contactHref(contact) {
    if (!contact || typeof contact !== "object") return null;
    let { type, value } = contact;
    if (typeof value !== "string" || /[\r\n\u0000]/.test(value)) return null;
    value = value.trim();
    if (type === "email" && /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
        return `mailto:${encodeURIComponent(value)}`;
    }
    if (type === "phone" && /^[+0-9 ().-]{3,80}$/.test(value) && /\d{3}/.test(value.replace(/\D/g, ""))) {
        return `tel:${value.replace(/[ ().-]/g, "")}`;
    }
    if (type === "url") {
        try {
            const url = new URL(value);
            if (["https:", "http:"].includes(url.protocol) && !url.username && !url.password) return url.href;
        } catch { /* Invalid stored values do not become clickable links. */ }
    }
    return null;
}
