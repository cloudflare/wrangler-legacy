# 🤠 wrangler

![Banner](/banner.png)

[![crates.io](https://meritbadge.herokuapp.com/wrangler)](https://crates.io/crates/wrangler) &nbsp;
[![Build Status](https://dev.azure.com/ashleygwilliams/wrangler/_apis/build/status/cloudflare.wrangler?branchName=master)](https://dev.azure.com/ashleygwilliams/wrangler/_build/latest?definitionId=1&branchName=master)

`wrangler` is a CLI tool designed for folks who are interested in using [Cloudflare Workers](https://workers.cloudflare.com/).

![Wrangler Demo](/wrangler-demo.gif)

## Installation

You have many options to install wrangler!

### Install with `npm`

```bash
npm i @cloudflare/wrangler -g
```

### Install with `cargo`

```bash
cargo install wrangler
```

If you don't have `cargo` or `npm` installed, you will need to follow these [additional instructions](#additional-installation-instructions).

## Updating

For information regarding updating Wrangler, click [here](https://workers.cloudflare.com/docs/quickstart/updating-the-cli/).

## Additional Documentation

All information regarding wrangler or Cloudflare Workers is located in the [Cloudflare Workers Developer Docs](https://developers.cloudflare.com/workers/). This includes:
* Using wrangler [commands](https://developers.cloudflare.com/workers/tooling/wrangler/commands) 
* Wrangler [configuration](https://developers.cloudflare.com/workers/tooling/wrangler/configuration)
* General documentation surrounding Workers development
* All wrangler features such as Workers Sites and KV
  

This documentation will be highly valuable to you when developing with `wrangler`.

#### ✨Workers Sites

To learn about deploying static assets using `wrangler`, see the [Workers Sites Quickstart](https://developers.cloudflare.com/workers/sites/).
