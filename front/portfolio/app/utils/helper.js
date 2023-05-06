import { QuillDeltaToHtmlConverter } from 'quill-delta-to-html';
export const secureImageSrc = src => src ? src.replace(/".*/g, '') : src

export const htmlDescription = description => {
    if (description) {
        const converter = new QuillDeltaToHtmlConverter(description.ops, {});
        const html = converter.convert();
        console.log(html);
        return html;
    }
    return '';
}