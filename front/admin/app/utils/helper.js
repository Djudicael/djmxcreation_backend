export const secureImageSrc = src => src ? src.replace(/".*/g, '') : src;

export const editorConfig = {
    theme: 'snow',
    modules: {
        toolbar: [
            ['bold', 'italic', 'underline', 'strike'],    // toggled buttons
            ['blockquote', 'code-block'],
            [{ 'header': 1 }, { 'header': 2 }],           // custom button values
            [{ 'list': 'ordered' }, { 'list': 'bullet' }],
            [{ 'script': 'sub' }, { 'script': 'super' }], // superscript/subscript
            [{ 'indent': '-1' }, { 'indent': '+1' }],     // outdent/indent
            [{ 'direction': 'rtl' }],                     // text direction

            [{ 'size': ['small', false, 'large', 'huge'] }],   // custom dropdown
            [{ 'header': [1, 2, 3, 4, 5, 6, false] }],

            [{ 'color': [] }, { 'background': [] }],      // dropdown with defaults from theme
            [{ 'font': [] }],
            [{ 'align': [] }],

            ['clean'],                                     // remove formatting button

            [{ 'video': true }],                          // add video option
            [{ 'image': true }],                          // add image option
            [{ 'formula': true }],                        // add formula option
            [{ 'link': { tooltip: 'Insert link' } }],
            // [{ 'emoji': true }],                          // add emoji option
            // [{ 'fullscreen': true }],
            // [{ 'table': true }],
            [{ 'code-block': 'code' }],                   // add code option
        ]
    }
}
