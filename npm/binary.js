const { Binary } = require("binary-install");
const os = require("os");
const { join } = require("path");

const getPlatform = () => {
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
};

const getBinaryURL = (version, platform) => {
  const site = process.env.WRANGLER_BINARY_HOST ||
      process.env.npm_config_wrangler_binary_host ||
      'https://workers.cloudflare.com/get-npm-wrangler-binary';
  return `${site}/${version}/${platform}`;
};

const getBinary = () => {
  const platform = getPlatform();
  const version = require("./package.json").version;
  const url = getBinaryURL(version, platform);
  const installDirectory = join(os.homedir(), ".wrangler");
  return new Binary(url, { name: "wrangler", installDirectory });
};

const run = () => {
  const binary = getBinary();
  binary.run();
};

const install = () => {
  const binary = getBinary();
  binary.install();
};

const uninstall = () => {
  const binary = getBinary();
  binary.uninstall();
}

module.exports = {
  install,
  run,
  uninstall
};
