const axios = require("axios");
const os = require("os");
const { join, resolve } = require("path");
const { mkdirSync, existsSync } = require("fs");
// while recent versions of Node can do that natively, wait until we can use it.
const rimraf = require("rimraf");
const tar = require("tar");
const { get } = axios;
const { homedir } = require('os');
const envPaths = require('env-paths');

let cwd;
const legacyCwd = join(homedir(), ".wrangler");
const newCwd = envPaths('wrangler', { suffix: '' }).config;

function getLatestRelease() {
  return get("https://api.github.com/repos/cloudflare/wrangler/releases/latest")
    .then(res => get(res.data.assets_url))
    .then(res => res.data);
}

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

function downloadAsset(asset) {
  const dest = join(cwd, "out");

  if (existsSync(dest)) {
    rimraf.sync(dest);
  }
  mkdirSync(dest);

  console.log("Downloading release", asset.browser_download_url);

  return axios({
    url: asset.browser_download_url,
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

if (existsSync(legacyCwd)) {
  cwd = legacyCwd;
} else {
  cwd = newCwd;

  if (!existsSync(cwd)) {
    mkdirSync(cwd);
  }
}

getLatestRelease()
  .then(assets => {
    const [compatibleAssets] = assets.filter(asset =>
      asset.name.endsWith(getPlatform() + ".tar.gz")
    );

    if (compatibleAssets === undefined) {
      throw new Error("No compatible release has been found");
    }

    return downloadAsset(compatibleAssets);
  })
  .then(() => {
    console.log("Wrangler has been installed!");
  })
  .catch(err => {
    throw err;
  });
