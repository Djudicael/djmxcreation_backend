export const secureImageSrc = src => src ? src.replace(/".*/g, "") : src;

export const editorConfig = {
    theme: "snow",
    placeholder: "Write your text…",
    modules: {
        toolbar: [
            [{ header: [2, 3, false] }],
            ["bold", "italic", "underline"],
            [{ list: "ordered" }, { list: "bullet" }],
            ["blockquote", "link", "clean"],
        ],
    },
};
