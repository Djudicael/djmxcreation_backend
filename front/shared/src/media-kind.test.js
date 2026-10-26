import test from "node:test";
import assert from "node:assert/strict";
import { mediaKind } from "./media-kind.js";

test("legacy media with signed URLs retain image and video previews", () => {
    assert.equal(mediaKind({ mimeType: null, url: "https://storage.example/bucket/Photo.PNG?signature=video.mp4" }), "image");
    assert.equal(mediaKind({ url: "https://storage.example/bucket/reel.mp4?signature=image.png" }), "video");
    assert.equal(mediaKind({ mimeType: "image/png", url: "https://storage.example/no-extension" }), "image");
    assert.equal(mediaKind({ url: "https://storage.example/manual.pdf?name=photo.png" }), "file");
});
