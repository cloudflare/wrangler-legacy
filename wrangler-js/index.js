#!/usr/bin/env node

const webpack = require("webpack");
const { ConcatSource } = require("webpack-sources");
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
    wasm_size: 0,
    wasm_name: "",
    script: null,
    script_size: 0,
    dist_to_clean: fullConfig.output.path,
  };

  const wasmModuleAsset = Object.keys(assets).find(filterByExtension("wasm"));
  const jsAssets = Object.keys(assets).filter(filterByExtension("js"));
  const hasWasmModule = wasmModuleAsset !== undefined;

  const script = jsAssets.reduce((acc, k) => {
    acc.add(assets[k]);
    return acc;
  }, new ConcatSource(""));
  bundle.script = script.source();
  bundle.script_size = script.size();

  if (hasWasmModule === true) {
    bundle.wasm = Buffer.from(assets[wasmModuleAsset].source()).toString();
    // TODO(sven): there is an issue in {Webpack} to retrive binary size
    if (assets[wasmModuleAsset].size() !== undefined) {
      bundle.wasm_size = assets[wasmModuleAsset].size();
    }
    bundle.wasm_name = wasmModuleAsset;
  }

  writeFileSync(args["output-file"], JSON.stringify(bundle));
});
