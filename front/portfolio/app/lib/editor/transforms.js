const alignType = ["left", "right", "center", "justify"]

const transforms = {
    delimiter: () => {
        return `<br/>`
    },

    header: ({ data }) => {
        return `<h${data.level}>${data.text}</h${data.level}>`
    },

    paragraph: ({ data }) => {
        const paragraphAlign = data.alignment || data.align

        if (
            typeof paragraphAlign !== "undefined" &&
            alignType.includes(paragraphAlign)
        ) {
            return `<p style="text-align:${paragraphAlign};">${data.text}</p>`
        } else {
            return `<p>${data.text}</p>`
        }
    },

    list: ({ data }) => {
        const listStyle = data.style === "unordered" ? "ul" : "ol"

        const recursor = (items, listStyle) => {
            const list = items.map(item => {
                if (!item.content && !item.items) return `<li>${item}</li>`

                let list = ""
                if (item.items) list = recursor(item.items, listStyle)
                if (item.content) return `<li> ${item.content} </li>` + list
            })

            return `<${listStyle}>${list.join("")}</${listStyle}>`
        }

        return recursor(data.items, listStyle)
    },

    image: ({ data }) => {
        let caption = data.caption ? data.caption : "Image"
        return `<img src="${data.file && data.file.url ? data.file.url : data.url
            }" alt="${caption}" />`
    },

    quote: ({ data }) => {
        return `<blockquote>${data.text}</blockquote> - ${data.caption}`
    },

    code: ({ data }) => {
        return `<pre><code>${data.code}</code></pre>`
    },

    embed: ({ data }) => {
        switch (data.service) {
            case "vimeo":
                return `<iframe src="${data.embed}" height="${data.height}" frameborder="0" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>`
            case "youtube":
                return `<iframe width="${data.width}" height="${data.height}" src="${data.embed}" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>`
            default:
                throw new Error("Only Youtube and Vime Embeds are supported right now.")
        }
    },
    table: ({ data }) => {
        let html = '<table>'
        html += '<thead><tr>'
        data.content[0].forEach((heading) => {
            html += `<th>${heading}</th>`
        })
        html += '</tr></thead><tbody>'
        data.content.slice(1).forEach((row) => {
            html += '<tr>'
            row.forEach((cell) => {
                html += `<td>${cell}</td>`
            })
            html += '</tr>'
        })
        html += '</tbody></table>'
        return html
    },

}

export default transforms
