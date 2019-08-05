module.exports = {
    entry: "./index.js",
    module: {
        rules: [
            {
                test: /\.wast$/,
                loader: "wast-loader",
                type: "webassembly/experimental"
            }
        ]
    },
}
