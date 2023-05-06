import transforms from "./transforms.js"

const ParseFunctionError = (errorType) => {
    return new Error(`Function ${errorType} is not defined`);
}

const parser = (plugins = {}) => {
    const parsers = Object.assign({}, transforms, plugins)

    return {
        parse: (blocks) => {

            return blocks.map(block => {
                return parsers[block.type]
                    ? parsers[block.type](block)
                    : ParseFunctionError(block.type)
            })
        },

        parseBlock: block => {
            console.log(block);
            return parsers[block.type]
                ? parsers[block.type](block)
                : ParseFunctionError(block.type)
        },

        parseStrict: ({ blocks }) => {
            const parserFreeBlocks = parser(parsers).validate({ blocks })

            if (parserFreeBlocks.length) {
                throw new Error(
                    `Parser Functions missing for blocks: ${parserFreeBlocks.toString()}`
                )
            }

            const parsed = []

            for (let i = 0; i < blocks.length; i++) {
                if (!parsers[blocks[i].type]) throw ParseFunctionError(blocks[i].type)

                parsed.push(parsers[blocks[i].type](blocks[i]))
            }

            return parsed
        },

        validate: ({ blocks }) => {

            console.log(blocks);
            const types = blocks
                .map(item => item.type)
                .filter((item, index, blocksArr) => blocksArr.indexOf(item) === index)

            const parser_keys = Object.keys(parsers)

            return types.filter(type => !parser_keys.includes(type))
        }
    }
}

export default parser
