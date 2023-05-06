export default class EditorJSRenderer {
    constructor() {
        this.blocks = {
            paragraph: this.renderParagraph,
            header: this.renderHeader,
            list: this.renderList,
            table: this.renderTable,
            quote: this.renderQuote,
            code: this.renderCode,
            embed: this.renderEmbed,
            image: this.renderImage,
        };
    }

    output(blocks) {
        return blocks.map((block) => {
            const renderer = this.blocks[block.type];
            if (renderer) {
                return renderer.call(this, block.data);
            }
            return null;
        }).join('');
    }

    renderParagraph(data) {
        const align = data.alignment || data.align;
        return `<p${align ? ` style="text-align:${align}"` : ''}>${data.text}</p>`;
    }

    renderHeader(data) {
        return `<h${data.level}>${data.text}</h${data.level}>`;
    }

    renderList(data) {
        const tag = data.style === 'unordered' ? 'ul' : 'ol';
        const items = data.items.map(item => `<li>${item}</li>`).join('');
        return `<${tag}>${items}</${tag}>`;
    }

    renderTable(data) {
        const headers = data.content[0].map((header) => `<th>${header}</th>`).join('');
        const rows = data.content.slice(1).map((row) => {
            const cells = row.map((cell) => `<td>${cell}</td>`).join('');
            return `<tr>${cells}</tr>`;
        }).join('');
        return `<table><thead><tr>${headers}</tr></thead><tbody>${rows}</tbody></table>`;
    }

    renderQuote(data) {
        return `<blockquote>${data.text}</blockquote>`;
    }

    renderCode(data) {
        return `<pre><code>${data.code}</code></pre>`;
    }

    renderEmbed(data) {
        return `<div class="embed-container">${data.embed}</div>`;
    }

    renderImage(data) {
        return `<img src="${data.file.url}" alt="${data.caption}"${data.stretched ? ' style="width:100%"' : ''}>`;
    }
}
