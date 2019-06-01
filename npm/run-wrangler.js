#!/usr/bin/env node

const { join, resolve } = require("path");
const { spawnSync } = require("child_process");
const { homedir } = require('os');

const cwd = join(homedir(), ".wrangler");
const bin = join(cwd, "out", "wrangler");
const [, , ...args] = process.argv;

const opts = {
  cwd: process.cwd(),
  stdio: "inherit"
};
spawnSync(bin, args, opts);
