#!/usr/bin/env node

const webpack = require("webpack");
const { join } = require("path");

function hasFlag(name) {
  return process.argv.indexOf("--" + name) !== -1;
}

let config;
if (hasFlag("auto-webpack-config") === true) {
  const entry = require(join(process.cwd(), "package.json")).main;
  config = { entry };
} else {
  config = require(join(process.cwd(), "./webpack.config.js"));
}

let compilerOutput = "";
const oldConsoleLog = console.log;
console.log = (...msg) => compilerOutput += msg.join(" ");

const compiler = webpack(config);

function filterByExtension(ext) {
  return v => v.indexOf("." + ext) !== -1;
}

function emitForWrangler(assets) {
  const bundle = {
    wasm: null,
    wasm_name: "",
    script: null,
    compiler_output: compilerOutput,
  };

  const wasmModuleAsset = Object.keys(assets).find(filterByExtension("wasm"));
  const jsAssets = Object.keys(assets).filter(filterByExtension("js"));
  const hasWasmModule = wasmModuleAsset !== undefined;

  bundle.script =
    jsAssets.reduce((acc, k) => {
      const asset = assets[k];
      return acc + asset.source();
    }, "");

  if (hasWasmModule === true) {
    bundle.wasm = Buffer.from(assets[wasmModuleAsset].source()).toString();
    bundle.wasm_name = wasmModuleAsset;
  }

  console.log(JSON.stringify(bundle));
}

compiler.run((err, stats) => {
  if (err) {
    throw err;
  }

  console.log =  oldConsoleLog;
  emitForWrangler(stats.compilation.assets);
  // console.log(stats.toString({ colors: true }));
});
