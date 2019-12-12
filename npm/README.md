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

If you don't have `cargo` or `npm` installed, you will need to follow these [additional instructions](https://developers.cloudflare.com/workers/tooling/wrangler/install/).

## Updating

For information regarding updating Wrangler, click [here](https://workers.cloudflare.com/docs/quickstart/updating-the-cli/).

## Getting Started

Once you have installed Wrangler, spinning up and deploying your first Worker is easy!

```console
$ wrangler generate my-worker
$ cd my-worker
# update your wrangler.toml with your Cloudflare Account ID
$ wrangler config
$ wrangler publish
```

## 🎙️ Top Level Commands

### 👯 `generate`

Scaffold a project, including boilerplate code for a Rust library and a Cloudflare Worker.

```bash
wrangler generate <name> <template> --type=["webpack", "javascript", "rust"]
```

All of the arguments and flags to this command are optional:

- `name`: defaults to `worker`
- `template`: defaults to the [`https://github.com/cloudflare/worker-template`](https://github.com/cloudflare/worker-template)
- `type`: defaults to ["webpack"](https://developers.cloudflare.com/workers/tooling/wrangler/webpack)

### 📥 `init`

Creates a skeleton `wrangler.toml` in an existing directory. This can be used as an alternative to `generate` if you prefer to clone a repository yourself.

```bash
wrangler init <name> --type=["webpack", "javascript", "rust"]
```

All of the arguments and flags to this command are optional:

- `name`: defaults to the name of your working directory
- `type`: defaults to ["webpack"](https://developers.cloudflare.com/workers/tooling/wrangler/webpack).

### 🦀⚙️ `build`

Build your project. This command looks at your `wrangler.toml` file and runs the build steps associated
with the `"type"` declared there.

Additionally, you can configure different [environments](https://developers.cloudflare.com/workers/tooling/wrangler/configuration/environments).

### 🔧 `config`

Configure your global Cloudflare user. This is an interactive command that will prompt you for your API token:

```bash
wrangler config
Enter API token:
superlongapitoken
```

You can also provide your email and global API key (this is not recommended for security reasons):

```bash
wrangler config --api-key
Enter email:
testuser@example.com
Enter global API key:
superlongapikey
```

You can also [use environment variables](https://developers.cloudflare.com/workers/tooling/wrangler/configuration/) to configure these values.

### ☁️ 🆙 `publish`

Publish your Worker to Cloudflare. Several keys in your `wrangler.toml` determine whether you are publishing to a workers.dev subdomain or your own registered domain, proxied through Cloudflare.

Additionally, you can configure different [environments](https://developers.cloudflare.com/workers/tooling/wrangler/configuration/environments).

You can also use environment variables to handle authentication when you publish a Worker.

    ```bash
    # e.g.
    CF_API_TOKEN=superlongtoken wrangler publish
    # where
    # $CF_API_TOKEN -> your Cloudflare API token

    CF_API_KEY=superlongapikey CF_EMAIL=testuser@example.com wrangler publish
    # where
    # $CF_API_KEY -> your Cloudflare API key
    # $CF_EMAIL -> your Cloudflare account email
    ```

### 🗂 `kv`

Interact with your Workers KV store. This is actually a whole suite of subcommands. Read more about in [Wrangler KV Documentation](https://developers.cloudflare.com/workers/tooling/wrangler/kv_commands).

## Additional Documentation

All information regarding wrangler or Cloudflare Workers is located in the [Cloudflare Workers Developer Docs](https://developers.cloudflare.com/workers/). This includes:

- Using wrangler [commands](https://developers.cloudflare.com/workers/tooling/wrangler/commands)
- Wrangler [configuration](https://developers.cloudflare.com/workers/tooling/wrangler/configuration)
- General documentation surrounding Workers development
- All wrangler features such as Workers Sites and KV

#### ✨Workers Sites

To learn about deploying static assets using `wrangler`, see the [Workers Sites Quickstart](https://developers.cloudflare.com/workers/sites/).
