#!/usr/bin/env node

const webpack = require("webpack");
const { join } = require("path");
const { writeFileSync } = require("fs");

const rawArgs = process.argv.slice(2);
const args = rawArgs.reduce((obj, e) => {
  if (e.indexOf("--") === -1 && e.indexOf("=") === -1) {
    throw new Error("malformed arguments");
  }

  const [name, value] = e.split("=");
  const normalizedName = name.replace("--", '');
  obj[normalizedName] = value;
  return obj;
}, {});

let config;
if (args["no-webpack-config"] === "1") {
  config = { entry: args["use-entry"] };
} else {
  config = require(join(process.cwd(), "./webpack.config.js"));
}

if (config.target !== "webworker") {
  if (config.target !== undefined) {
    console.warn("Unsupported Webpack target; webworker has been set.");
  }
  config.target = "webworker";
}
const compiler = webpack(config);

function filterByExtension(ext) {
  return v => v.indexOf("." + ext) !== -1;
}

compiler.run((err, stats) => {
  if (err) {
    throw err;
  }

  const fullConfig = compiler.options;
  const assets = stats.compilation.assets;
  const bundle = {
    wasm: null,
    wasm_name: "",
    script: null,
    dist_to_clean: fullConfig.output.path,
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
    bundle.wasm_name = wasmModuleAsset;
  }

  writeFileSync(args["output-file"], JSON.stringify(bundle));

  // FIXME(sven): stats could be printed in {wrangler}, avoiding any confusion.
  console.log(stats.toString({ colors: true }));
});
