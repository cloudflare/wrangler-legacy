const { existsSync } = require("fs");
const { homedir } = require("os");
const { join } = require("path");
const { spawnSync } = require("child_process");
const { URL } = require("universal-url");
const envPaths = require("env-paths");
const mkdirp = require("mkdirp");

const axios = require("axios");
const tar = require("tar");
const rimraf = require("rimraf");

class Binary {
  constructor(url, data) {
    if (typeof url !== "string") {
      errors.push("url must be a string");
    } else {
      try {
        new URL(url);
      } catch (e) {
        errors.push(e);
      }
    }
    let errors = [];
    if (data.name && typeof data.name !== "string") {
      errors.push("name must be a string");
    }
    if (data.installDirectory && typeof data.installDirectory !== "string") {
      errors.push("installDirectory must be a string");
    }
    if (!data.installDirectory && !data.name) {
      errors.push("You must specify either name or installDirectory");
    }
    if (errors.length > 0) {
      console.error("Your Binary constructor is invalid:");
      errors.forEach(error => {
        console.error(error);
      });
    }
    this.url = url;
    this.name = data.name || -1;
    this.installDirectory = data.installDirectory || envPaths(this.name).config;
    this.binaryDirectory = -1;
    this.binaryPath = -1;
  }

  _getInstallDirectory() {
    if (!existsSync(this.installDirectory)) {
      mkdirp.sync(this.installDirectory);
    }
    return this.installDirectory;
  }

  _getBinaryDirectory() {
    const installDirectory = this._getInstallDirectory();
    const binaryDirectory = join(this.installDirectory, "bin");
    if (existsSync(binaryDirectory)) {
      this.binaryDirectory = binaryDirectory;
    } else {
      throw `You have not installed ${this.name ? this.name : "this package"}`;
    }
    return this.binaryDirectory;
  }

  _getBinaryPath() {
    if (this.binaryPath === -1) {
      const binaryDirectory = this._getBinaryDirectory();
      this.binaryPath = join(binaryDirectory, this.name);
    }

    return this.binaryPath;
  }

  install() {
    const dir = this._getInstallDirectory();
    if (!existsSync(dir)) {
      mkdirp.sync(dir);
    }

    this.binaryDirectory = join(dir, "bin");

    if (existsSync(this.binaryDirectory)) {
      rimraf.sync(this.binaryDirectory);
    }

    mkdirp.sync(this.binaryDirectory);

    console.log("Downloading release", this.url);

    return axios({
      url: this.url,
      responseType: "stream"
    })
      .then(res => {
        res.data.pipe(
          tar.x({
            strip: 1,
            C: this.binaryDirectory
          })
        );
      })
      .then(() => {
        console.log(
          `${this.name ? this.name : "Your package"} has been installed!`
        );
      })
      .catch(e => {
        console.error("Error fetching release", e.message);
        throw e;
      });
  }

  uninstall() {
    if (existsSync(this._getInstallDirectory())) {
      rimraf.sync(this.installDirectory);
      console.log(
        `${this.name ? this.name : "Your package"} has been uninstalled`
      );
    }
  }

  run() {
    const binaryPath = this._getBinaryPath();
    const [, , ...args] = process.argv;

    const options = {
      cwd: process.cwd(),
      stdio: "inherit"
    };

    const result = spawnSync(binaryPath, args, options);

    if (result.error) {
      console.error(result.error);
      process.exit(1);
    }

    process.exit(result.status);
  }
}

module.exports = Binary;
