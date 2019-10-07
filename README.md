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

General documentation surrounding Workers development and using `wrangler` can be found [here](https://developers.cloudflare.com/workers). This documentation will be highly valuable to you when developing with `wrangler`.

#### ✨Workers Sites

To learn about deploying static assets using `wrangler`, see the [Workers Sites Quickstart](https://developers.cloudflare.com/workers/sites/reference/).

## 🎙️ Commands

- ### 👯 `generate`

  Scaffold a project, including boilerplate for a Rust library and a Cloudflare Worker.
  You can pass a name and template to this command optionally.

  ```bash
  wrangler generate <name> <template> --type=["webpack", "javascript", "rust"]
  ```

  All of the arguments and flags to this command are optional:
  
  - `name`: defaults to `worker`
  - `template`: defaults to the [`https://github.com/cloudflare/worker-template`](https://github.com/cloudflare/worker-template)
  - `type`: defaults to ["webpack"](https://github.com/cloudflare/wrangler/blob/master/docs/content/webpack.md)

- ### 📥 `init`

  Creates a skeleton `wrangler.toml` in an existing directory. This can be used as an alternative to `generate` if you prefer to clone a repository yourself.

  ```bash
  wrangler init <name> --type=["webpack", "javascript", "rust"]
  ```

  All of the arguments and flags to this command are options:

  - `name`: defaults to the name of your working directory
  - `type`: defaults to ["webpack"](https://github.com/cloudflare/wrangler/blob/master/docs/content/webpack.md).

- ### 🦀⚙️ `build`

  Build your project. This command looks at your `wrangler.toml` file and runs the build steps associated
  with the `"type"` declared there.

  Additionally, you can build different environments. This is useful if you have different builds for different environments, but typically isn't needed. For more information see the [environments documentation](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md).

- ### 🔧 `config`

  Configure your global Cloudflare user. This is an interactive command that will prompt you for your email and API key:

  ```bash
  wrangler config
  Enter email:
  testuser@example.com
  Enter api key:
  ...
  ```

  You can also [use environment variables](#using-environment-variables) to configure these values.

- ### ☁️ 🆙 `publish`

  Publish your Worker to Cloudflare. Several keys in your `wrangler.toml` determine whether you are publishing to a workers.dev subdomain or your own registered domain, proxied through Cloudflare.

  ```bash
  wrangler publish
  ```

  To use this command, the following fields are required in your `wrangler.toml`.

  | Key        | Value                                                                     | Example                                           |
  | ---------- | ------------------------------------------------------------------------- | ------------------------------------------------- |
  | name       | the name of your worker                                                   | `name = "your-worker"`                            |
  | type       | build type (webpack, rust, or javascript)                                 | `type = "webpack"`                                |
  | account_id | your Cloudflare account ID, this can be found in the Cloudflare dashboard | `account_id = "a655bacaf2b4cad0e2b51c5236a6b974"` |

  From here, you have two options, you can choose to publish to your own domain or you can choose to publish to [\<your-worker\>.\<your-subdomain\>.workers.dev](https://workers.dev).

#### Publishing to workers.dev

  If you want to publish to [workers.dev](https://workers.dev), you will first need to have a [workers.dev](https://workers.dev) subdomain registered. You can register a subdomain by executing:

  ```bash
  wrangler subdomain <name>
  ```

  After you have registered a subdomain, add `workers_dev` to your `wrangler.toml`.

| Key             | Value | Example                  |
| --------------- | ----- | ------------------------ |
| workers_dev | true  | `workers_dev = true` |

#### Publishing to your own domain

If you would like to publish to your own domain, you will need to specify these three fields in your `wrangler.toml`.

| Key             | Value                                                                  | Example                                        |
| --------------- | ---------------------------------------------------------------------- | ---------------------------------------------- |
| workers_dev | false                                                                  | `workers_dev = false`                      |
| route           | The route you would like to publish to                                 | `route = "example.com/my-worker/*"`            |
| zone_id         | Your Cloudflare zone ID, this can be found in the Cloudflare dashboard | `zone_id = "b6558acaf2b4cad1f2b51c5236a6b972"` |

#### Publishing the same code to multiple places

If you would like to be able to publish your code to multiple places, please see the documentation for [environments](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md).

- ### 🔬 `preview`

  Preview your project using the [Cloudflare Workers preview service](https://cloudflareworkers.com/).

  By default, `wrangler preview` will only bundle your project a single time. To enable live preview,
  where Wrangler will continually update the preview service with the newest version of your project,
  pass the `--watch` flag:

  ```bash
  wrangler preview --watch
  ```

  You can optionally pass `get` or `post` and a `body` to this command. This will send a request to your
  worker on the preview service and return the response in your terminal. For example:

  GET requests can be sent with

  ```bash
  wrangler preview
  ```

  or

  ```bash
  wrangler preview get
  ```

  POST requests can be sent with

  ```bash
  wrangler preview post hello=hello
  ```

  Additionally, you can preview different environments. This is useful if you have different builds for different environments (like staging vs. production), but typically isn't needed. For more information see the [environments documentation](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md).

- ### 🗂️ `kv`

  Interact with your Cloudflare Workers KV store. [Check out the docs.](./docs/content/kv_commands.md)

## 🔩 Configuration

There are two types of configuration that `wrangler` uses: global user and per project.

- ### Global User

  In Cloudflare's system, you have a User that can have multiple Accounts and Zones. As a result, your User
  is configured globally on your machine. Your Account(s) and Zone(s) will be configured per project, but
  will use your User credentials to authenticate all API calls. This config file is created in a `.wrangler`
  directory in your computer's home directory.

  To set up `wrangler` to work with your Cloudflare user, use the following commands:

  - 🔧 `config`: a command that prompts you to enter your `email` and `api` key.
  - 🕵️‍♀️ `whoami`: run this command to confirm that your configuration is appropriately set up.
    When successful, this command will print out your user information, including the type of plan you
    are currently on.

- #### Using environment variables

  You can also configure your global user with environment variables. This is the preferred method for using Wrangler in CI:

  ```bash
  # e.g.
  CF_API_KEY=superlongapikey CF_EMAIL=testuser@example.com wrangler publish --release
  # where
  # $CF_API_KEY -> your Cloudflare API key
  # $CF_EMAIL -> your Cloudflare account email
  ```

- ### Per Project

    Your project will need to have several things configured before you can publish your worker. These values are stored in a `wrangler.toml` file that `wrangler generate` will make for you. You will need to manually edit this file to add these values before you can publish.

    - `name`: This is the name of your project. It will be the name of your script.
    - `type`: This key tells `wrangler build` how to build your project. There are currently three options (`webpack`, `javascript`, and `rust`), but we expect there to be more as the community grows.
        - `javascript`: This project contains a single JavaScript file, defined in `package.json`'s `main` key.
        - `rust`: This project contains a Rust crate that uses `wasm-bindgen`. It will be built with `wasm-pack`.
        - `webpack`: This project contains any number of JavaScript files or Rust/C/C++ files that compile to
            WebAssembly. Rust files will be built with `wasm-pack`.
            This project type uses webpack and webpack plugins in the background to build your worker. You can read more about this type [here](https://github.com/cloudflare/wrangler/blob/master/docs/content/webpack.md).
    - `zone_id`: This is the ID of the "zone" or domain you want to run your script on. This is optional if you are using a [workers.dev](https://workers.dev) subdomain and is only required when `workers_dev` is false, or excluded from an [environment](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md) configuration.
    - `account_id`: This is the ID of the account associated with your zone. You might have more than one account, so make sure to use the ID of the account associated with the `zone_id` you provide, if you provide one.
    - `route`: This is the route you'd like to use your worker on. You need to include the hostname. Examples:

        - `*example.com/*`
        - `http://example.com/hello`
        
        This key is optional if you are using a [workers.dev](https://workers.dev) subdomain and is only required when `workers_dev` is false, or excluded from an [environment](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md). 

    - `webpack_config`: This is the path to a custom webpack configuration file for your worker. You must specify this field to use a custom webpack configuration, otherwise Wrangler will use a default configuration for you. You can read more [here](https://github.com/cloudflare/wrangler/blob/master/docs/content/webpack.md).
    - `workers_dev`: This is a boolean flag that specifies if your worker will be deployed to your [workers.dev](https://workers.dev) subdomain. For more information, please read the [environments documentation](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md).
    - `kv-namespaces`: These specify any [Workers KV](https://workers.cloudflare.com/docs/reference/storage/) Namespaces you want to access from
        inside your Worker. Each namespace you include should have an entry in your `wrangler.toml` that includes:

        - `binding`: the name you want to bind to in your script
        - `id`: the namespace_id assigned to your KV Namespace upon creation.

        For example:

        ```toml
        kv-namespaces = [
            { binding = "FOO", id = "0f2ac74b498b48028cb68387c421e279" },
            { binding = "BAR", id = "068c101e168d03c65bddf4ba75150fb0" }
        ]
        ```

        Note: Creating your KV Namespaces should be handled using Wrangler's [KV Commands](./docs/content/kv_commands.md).

    #### Environments

    Additionally, you can configure Wrangler to publish to multiple environments. This means that your same codebase can be deployed to multiple places on your [workers.dev](https://workers.dev) subdomain, across multiple accounts, zones, and routes. Read more [here](/docs/content/environments.md).

## Additional Installation Instructions

Wrangler can be installed both through [npm](https://www.npmjs.com/get-npm) and through Rust's package manager, [Cargo](https://github.com/rust-lang/cargo).

### Using `npm`

1. If you don't already have npm on your machine, install it using [npm's recommended method](https://www.npmjs.com/get-npm), a node.js version manager.

   If you have already installed npm with a package manager, it is possible you will run into an `EACCES` error while installing wrangler. This is related to how many system packagers install npm. You can either uninstall npm and reinstall using the npm recommended install method (a version manager), or use one of our other install methods.

2. Install Wrangler by running:

   ```bash
   npm i @cloudflare/wrangler -g
   ```

### Using `cargo`

1. Install `cargo`:

   Rustup, a tool for installing Rust, will also install Cargo. On Linux and macOS systems, `rustup` can be installed as follows:

   ```bash
   curl https://sh.rustup.rs -sSf | sh
   ```

   Additional installation methods are available [here](https://forge.rust-lang.org/other-installation-methods.html).

2. Install `wrangler`:

   ```bash
   cargo install wrangler
   ```

   Installing wrangler on linux requires some [OpenSSL-related packages](https://docs.rs/openssl/0.10.24/openssl/#automatic) to be installed. If you don't want to deal with this, you can use vendored OpenSSL.

   ```bash
   cargo install wrangler --features vendored-openssl
   ```

### Manual Install

1. Download the binary tarball for your platform from our [releases page](https://github.com/cloudflare/wrangler/releases). You don't need to download wranglerjs, wrangler will install that for you.

2. Unpack the tarball and place the binary `wrangler` somewhere on your `PATH`, preferably `/usr/local/bin` for linux/macOS or `Program Files` for windows.
