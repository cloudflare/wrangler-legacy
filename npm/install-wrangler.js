const axios = require("axios");
const os = require("os");
const { join, resolve } = require("path");
const { mkdirSync, existsSync } = require("fs");
// while recent versions of Node can do that natively, wait until we can use it.
const rimraf = require("rimraf");
const tar = require("tar");
const { get } = axios;
const { homedir } = require('os');

const cwd = join(homedir(), ".wrangler");

const VERSION = "1.5.0-rc.0"

function getPlatform() {
  const type = os.type();
  const arch = os.arch();

  if (type === "Windows_NT" && arch === "x64") {
    return "x86_64-pc-windows-msvc";
  }
  if (type === "Linux" && arch === "x64") {
    return "x86_64-unknown-linux-musl";
  }
  if (type === "Darwin" && arch === "x64") {
    return "x86_64-apple-darwin";
  }

  throw new Error(`Unsupported platform: ${type} ${arch}`);
}

function downloadAsset(version, platform) {
  const dest = join(cwd, "out");

  if (existsSync(dest)) {
    rimraf.sync(dest);
  }
  mkdirSync(dest);

  const url = `https://workers.cloudflare.com/get-npm-wrangler-binary/${ version }/${ platform }`

  console.log("Downloading release", url);

  return axios({
    url,
    responseType: "stream"
  }).then(res => {
    res.data.pipe(
      tar.x({
        strip: 1,
        C: dest
      })
    );
  });
}

if (!existsSync(cwd)) {
  mkdirSync(cwd);
}

downloadAsset(VERSION, getPlatform())
  .then(() => {
    console.log("Wrangler has been installed!");
  })
  .catch(e => {
    console.error("Error fetching release", e.message);
    throw e;
  });
