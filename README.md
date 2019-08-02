# 🤠 wrangler

![Banner](/banner.png)

[![crates.io](https://meritbadge.herokuapp.com/wrangler)](https://crates.io/crates/wrangler) &nbsp;
[![Build Status](https://dev.azure.com/ashleygwilliams/wrangler/_apis/build/status/cloudflare.wrangler?branchName=master)](https://dev.azure.com/ashleygwilliams/wrangler/_build/latest?definitionId=1&branchName=master)

`wrangler` is a CLI tool designed for folks who are interested in using [Cloudflare workers](https://workers.cloudflare.com/).

![Wrangler Demo](/wrangler-demo.gif)

## Installation

You have many options to install wrangler!

### Using `npm`

```
npm i @cloudflare/wrangler -g
```

### Using `cargo`

```
cargo install wrangler
```

## 🎙️ Commands

  - ### 👯 `generate`
    Scaffold a project, including boilerplate for a Rust library and a Cloudflare Worker.
    You can pass a name and template to this command optionally.

    ```
    wrangler generate <name> <template> --type=["webpack", "javascript", "rust"]
    ```

    All of the arguments and flags to this command are optional:
        - `name`: defaults to `worker`
        - `template`: defaults to the [`https://github.com/cloudflare/worker-template`](https://github.com/cloudflare/worker-template)
        - `type`: defaults to "webpack"

  - ### 🦀⚙️ `build`
    Build your project. This command looks at your `wrangler.toml` file and runs the build steps associated
    with the `"type"` declared there.

  - ### 🔧 `config`
    Configure your global Cloudflare user. This is an interactive command that will prompt you for your email and API key:

    ```
    wrangler config
    Enter email:
    ...
    Enter api key:
    ...
    ```

    You can also [use environment variables](#using-environment-variables) to configure these values.

  - ### ☁️ 🆙 `publish`

    Publish your Worker to Cloudflare. This uses several keys in your `wrangler.toml` depending on whether
    you are publishing to a workers.dev subdomain or your own domain, registered with Cloudflare.

    ```
    wrangler publish
    ```

    By default, `publish` will make your worker available at `<project-name>.<subdomain>.workers.dev`.
    To disable publishing to your workers.dev subdomain, set `private = true` in your `wrangler.toml`.
    This setting prevents the `publish` command from making your worker publicly available. To
    explicitly enable deployment to `<project-name>.<subdomain>.workers.dev`, you can set `private = false`.

    To use this command, you'll need to have the following keys in your `wrangler.toml`:

    - `name`
    - `type`
    - `account_id`

    You'll also need to have a workers.dev subdomain registered. You can register a subdomain by using:

    ```
    wrangler subdomain <name>
    ```

    A `--release` can be optionally passed to publish your worker to a domain you have registered with
    Cloudflare. To use `--release` your `wrangler.toml` must include:

    - `name`
    - `type`
    - `account_id`
    - `zone_id`
    - `route`

  - ### 🔬 `preview`
    Preview your project using the cloudflareworkers.com API.

    You can optionally pass `get` or `post` and a `body` to this command. This will send a request to your
    worker on the preview service and return the response in your terminal. For example:

    ```
    wrangler preview post hello=hello
    wrangler preview get // this is the default
    ```


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

	    ``` sh
	    # e.g.
	    CF_API_KEY=superlongapikey CF_EMAIL=user@mail.com wrangler publish --release
	    # where
	    # $CF_API_KEY -> your Cloudflare API key
	    # $CF_EMAIL -> your Cloudflare account email
	    ```

- ### Per Project

    Your project will need to have several things configured before you can publish your worker. These values
    are stored in a `wrangler.toml` file that `wrangler generate` will make for you. You will need to manually
    edit this file to add these values before you can publish.

    - `name`: This is the name of your project. It will be the name of your script.
    - `private`: This is a boolean. If set to `true`, when using `wrangler publish`, it will push your script but
        not make it publically available. This does not affect publishing in `--release` mode to a registered
        domain. Those pushes are *always* public. If this is not in your `wrangler.toml` it is assumed your
        project is public.
    - `type`: This key tells `wrangler build` how to build your project. There are currently 3 options, but we
        expect there to be more as the community grows.
        - `javascript`: This project contains a single JavaScript file, defined in `package.json`'s `main` key.
        - `rust`: This project contains a Rust crate that uses `wasm-bindgen`. It will be built with `wasm-pack`.
        - `webpack`: This project contains any number of JavaScript files or Rust/C/C++ files that compile to
            WebAssembly. Rust files will be built with `wasm-pack`.
            This project type uses webpack and webpack plugins in the background to build your worker.
    - `zone_id`: This is the ID of the "zone" or domain you want to run your script on. This is optional if you
        are using a workers.dev subdomain and is only reuqired for `publish --release`.
    - `account_id`: This is the ID of the account associated with your zone. You might have more than one account,
        so make sure to use the ID of the account associated with the `zone_id` you provide, if you provide one.
    - `route`: This is the route you'd like to use your worker on. You need to include the hostname. Examples:
        - `*example.com/*`
        - `http://example.com/hello`
        This key is optional if you are using a workers.dev subdomain and is only required for `publish --release`.
    - `webpack_config`: This is the path to the webpack configuration file for your worker. This is optional and
        defaults to `webpack.config.js`
    - `[[kv-namespaces]]`: These specify any [Workers KV](https://workers.cloudflare.com/docs/reference/storage/) namespaces you want to access from
        inside your Worker. Each namespace you include should have an entry in your wrangler.toml that includes:

        - `binding`: the name you want to bind to in your script
        - `id`: the namespace_id assigned to your kv namespace upon creation.
            e.g. (per namespace):
        ``` toml
        [[kv-namespaces]]
        binding = "FOO"
        id = "0f2ac74b498b48028cb68387c421e279"
        ```
        Note: Creating your KV Namespaces should be handled either via the [api](https://workers.cloudflare.com/docs/reference/storage/writing-data/) or via your Cloudflare dashboard.

## ⚓ Installation

Wrangler can be installed both through [npm](https://www.npmjs.com/get-npm) and through Rust's package manager, [Cargo](https://github.com/rust-lang/cargo).

### Using `npm`:

1. If you don't already have npm on your machine, install it using [npm's recommended method](https://www.npmjs.com/get-npm), a node.js version manager.

    If you have already installed npm with a package manager, it is possible you will run into an `EACCES` error while installing wrangler. This is related to how many system packagers install npm. You can either uninstall npm and reinstall using the npm recommended install method (a version manager), or use one of our other install methods.

1. Install Wrangler by running:

    ```
    npm i @cloudflare/wrangler -g
    ```

### Using `cargo`:

1. Install `cargo`:

    Rustup, a tool for installing Rust, will also install Cargo. On Linux and macOS systems, `rustup` can be installed as follows:

    ```
    curl https://sh.rustup.rs -sSf | sh
    ```

    Additional installation methods are available [here](https://forge.rust-lang.org/other-installation-methods.html).

1. Install `wrangler`:

    ```
    cargo install wrangler
    ```

### Manual Install:

1. Download the binary tarball for your platform from our [releases page](https://github.com/cloudflare/wrangler/releases). You don't need to download wranglerjs, wrangler will install that for you.

2. Unpack the tarball and place the binary `wrangler` somewhere on your `PATH`, preferably `/usr/local/bin` for linux/macOS or `Program Files` for windows.


## Updating `wrangler`:

   To get the latest version of Wrangler, using Cargo, run:

   ```
   cargo install wrangler --force
   ```

   To get the latest version of Wrangler, using NPM, run:

   ```
   npm install @cloudflare/wrangler
   ```

## ⚡ Quick Start

1. Generate a new project:

    ```
    wrangler generate
    ```

1. Move into the new project directory:

    ```
    cd worker
    ```

1. Build your project:

    ```
    wrangler build
    ```

1. Preview your project:

    ```
    wrangler preview
    ```

1. (optional) Configure with your Cloudflare account (or [use environment variables](#using-environment-variables)):

    ```
    wrangler config
    ```

    Configuring your account is required to use the `publish` step, which will push your Worker live to the
    Cloudflare edge. If you don't configure, you can still use `wrangler` to generate, build, and preview
    a Worker.

    Optionally, create a workers.dev subdomain:

    ```
    wrangler subdomain <name>
    ```

1. Check your configuration:

    ```
    wrangler whoami
    ```

1. Publish your project:

    To publish to a workers.dev subdomain:
    ```
    wrangler publish
    ```

    To publish to a domain you have registered with Cloudflare, add a `route` and a `zone_id` to your
    `wrangler.toml`. Then run:

    ```
    wrangler publish --release
    ```
