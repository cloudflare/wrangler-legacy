#!/usr/bin/env node

const { join } = require("path");
const { spawnSync } = require("child_process");
const { homedir } = require("os");

const cwd = join(homedir(), ".wrangler");
const bin = join(cwd, "out", "wrangler");
const [, , ...args] = process.argv;

const opts = {
  cwd: process.cwd(),
  stdio: "inherit"
};
process.exit(spawnSync(bin, args, opts).status);
