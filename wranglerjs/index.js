#!/usr/bin/env node

const webpack = require("webpack");
const { join } = require("path");
const { writeFileSync } = require("fs");
const WasmMainTemplatePlugin = require("webpack/lib/wasm/WasmMainTemplatePlugin");

const rawArgs = process.argv.slice(2);
const args = rawArgs.reduce((obj, e) => {
  if (e.indexOf("--") === -1 && e.indexOf("=") === -1) {
    throw new Error("malformed arguments");
  }

  const [name, value] = e.split("=");
  const normalizedName = name.replace("--", "");
  obj[normalizedName] = value;
  return obj;
}, {});

let config;
if (args["no-webpack-config"] === "1") {
  config = { entry: args["use-entry"] };
} else {
  config = require(join(process.cwd(), "./webpack.config.js"));
}

const compiler = webpack(config);
const fullConfig = compiler.options;

function filterByExtension(ext) {
  return v => v.indexOf("." + ext) !== -1;
}

// Override the {FetchCompileWasmTemplatePlugin} and inject our new runtime.
const [
  fetchCompileWasmTemplatePlugin
] = compiler.hooks.thisCompilation.taps.filter(
  tap => tap.name === "FetchCompileWasmTemplatePlugin"
);
fetchCompileWasmTemplatePlugin.fn = function(compilation) {
  const mainTemplate = compilation.mainTemplate;
  const generateLoadBinaryCode = () => `
      // Fake fetch response
      Promise.resolve({
        arrayBuffer() { return Promise.resolve(${args["wasm-binding"]}); }
      });
    `;

  const plugin = new WasmMainTemplatePlugin({
    generateLoadBinaryCode,
    mangleImports: false,
    supportsStreaming: false
  });
  plugin.apply(mainTemplate);
};

compiler.run((err, stats) => {
  if (err) {
    throw err;
  }

  const assets = stats.compilation.assets;
  const jsonStats = stats.toJson();
  const bundle = {
    wasm: null,
    script: "",
    dist_to_clean: fullConfig.output.path,
    errors: jsonStats.errors
  };

  const wasmModuleAsset = Object.keys(assets).find(filterByExtension("wasm"));
  const jsAssets = Object.keys(assets).filter(filterByExtension("js"));
  const hasWasmModule = wasmModuleAsset !== undefined;

  bundle.script = jsAssets.reduce((acc, k) => {
    const asset = assets[k];
    return acc + asset.source();
  }, "");

  if (hasWasmModule === true) {
    bundle.wasm = Buffer.from(assets[wasmModuleAsset].source()).toString();
  }

  writeFileSync(args["output-file"], JSON.stringify(bundle));
});
