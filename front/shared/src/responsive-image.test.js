import test from "node:test";
import assert from "node:assert/strict";
import { imageUrl, imageSrcset } from "./responsive-image.js";

test("missing, pending and failed optimization always use the original",()=>{
    for (const status of [undefined,"pending","processing","failed","skipped"]) {
        const media={url:"original.jpg",optimization:status ? {status,sources:[{url:"partial.jpg",width:400}]} : undefined};
        assert.equal(imageUrl(media,400),"original.jpg");assert.equal(imageSrcset(media),"");
    }
});
test("ready variants select adequate sizes, separate WebP and deduplicate widths",()=>{
    const media={url:"original.jpg",optimization:{status:"ready",sources:[
        {url:"small.jpg",width:400,mimeType:"image/jpeg"},
        {url:"large.jpg",width:1600,mimeType:"image/jpeg"},
        {url:"small.webp",width:400,mimeType:"image/webp"},
        {url:"large.webp",width:1600,mimeType:"image/webp"},
        {url:"small.jpg",width:400,mimeType:"image/jpeg"},
    ]}};
    assert.equal(imageUrl(media,400),"small.jpg");assert.equal(imageUrl(media,800),"large.jpg");assert.equal(imageUrl(media,2560),"large.jpg");
    assert.equal(imageSrcset(media),"small.jpg 400w, large.jpg 1600w");assert.equal(imageSrcset(media,true),"small.webp 400w, large.webp 1600w");
    media.optimization.sources=media.optimization.sources.filter(source=>source.url!=="large.webp");
    assert.equal(imageSrcset(media,true),"", "Partial WebP coverage must not reduce the available image resolution");
});
