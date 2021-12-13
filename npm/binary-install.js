const { existsSync, mkdirSync } = require("fs");
const { homedir } = require("os");
const { join } = require("path");
const { spawnSync } = require("child_process");

const axios = require("axios");
const tar = require("tar");
const rimraf = require("rimraf");

const error = msg => {
    console.error(msg);
    process.exit(1);
};

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
            let errorMsg = "Your Binary constructor is invalid:";
            errors.forEach(error => {
                errorMsg += error;
            });
            error(errorMsg);
        }
        this.url = url;
        this.name = data.name || -1;
        this.installDirectory = data.installDirectory || join(__dirname, "bin");
        this.binaryDirectory = -1;
        this.binaryPath = -1;
    }

    _getInstallDirectory() {
        if (!existsSync(this.installDirectory)) {
            mkdirSync(this.installDirectory, { recursive: true });
        }
        return this.installDirectory;
    }

    _getBinaryDirectory() {
        const installDirectory = this._getInstallDirectory();
        const binaryDirectory = join(this.installDirectory, "bin");
        if (existsSync(binaryDirectory)) {
            this.binaryDirectory = binaryDirectory;
        } else {
            error(`You have not installed ${this.name ? this.name : "this package"}`);
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
            mkdirSync(dir, { recursive: true });
        }

        this.binaryDirectory = join(dir, "bin");

        if (existsSync(this.binaryDirectory)) {
            rimraf.sync(this.binaryDirectory);
        }

        mkdirSync(this.binaryDirectory, { recursive: true });

        console.log(`Downloading release from ${this.url}`);

        return axios({ url: this.url, responseType: "stream" })
            .then(res => {
                const writer = tar.x({ strip: 1, C: this.binaryDirectory });

                return new Promise((resolve, reject) => {
                    res.data.pipe(writer);
                    let error = null;
                    writer.on('error', err => {
                      error = err;
                      reject(err);
                    });
                    writer.on('close', () => {
                      if (!error) {
                        resolve(true);
                      }
                    });
                })
            })
            .then(() => {
                console.log(
                    `${this.name ? this.name : "Your package"} has been installed!`
                );
            })
            .catch(e => {
                error(`Error fetching release: ${e.message}`);
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

        const options = { cwd: process.cwd(), stdio: "inherit" };

        const result = spawnSync(binaryPath, args, options);

        if (result.error) {
            error(result.error);
        }

        process.exit(result.status);
    }
}

module.exports.Binary = Binary;