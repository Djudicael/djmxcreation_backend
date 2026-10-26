import test from "node:test";
import assert from "node:assert/strict";
import { contactHref } from "./contact-links.js";

test("contact links support email, international phone and HTTP sites", () => {
    assert.equal(contactHref({ type: "email", value: "bonjour@example.com" }), "mailto:bonjour%40example.com");
    assert.equal(contactHref({ type: "phone", value: "+33 (0)6 12 34 56 78" }), "tel:+330612345678");
    assert.equal(contactHref({ type: "url", value: "https://example.com/me" }), "https://example.com/me");
});

test("malformed and executable stored links never become clickable", () => {
    for (const contact of [undefined, null, { type: "url", value: "javascript:alert(1)" }, { type: "url", value: "data:text/html,test" },
        { type: "url", value: "https://user:secret@example.com" }, { type: "email", value: "a@example.com\nBcc:other@example.com" },
        { type: "phone", value: "call me" }, { type: "email", value: "missing-at" }]) {
        assert.equal(contactHref(contact), null);
    }
});
