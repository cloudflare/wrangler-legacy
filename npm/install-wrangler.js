#!/usr/bin/env node

const { install } = require("./binary");

const deprecationMessage = `
                      ⛔   DEPRECATION   ⛔
The version of wrangler you are using is now deprecated. 

Please update to the latest version of wrangler to prevent critical errors.

Run \`npm uninstall -g @cloudflare/wrangler && npm install -g wrangler\` to update to the latest version
`;

install();
console.log(deprecationMessage);
