# 🤠 wrangler
![Banner](/banner.png)

[![crates.io](https://meritbadge.herokuapp.com/wrangler)](https://crates.io/crates/wrangler)
[![Build Status](https://dev.azure.com/ashleygwilliams/wrangler/_apis/build/status/cloudflare.wrangler?branchName=master)](https://dev.azure.com/ashleygwilliams/wrangler/_build/latest?definitionId=1&branchName=master)

✨ CHECK OUT THE [TUTORIAL](https://developers.cloudflare.com/workers/webassembly/tutorial/) ✨

`wrangler` is a CLI tool designed for folks who are interested in using Rust-generated WebAssembly on
Cloudflare Workers. This tool gives you the following commands:

  - ### 👯 `generate` 
    Scaffold a project, including boilerplate for a Rust library and a Cloudflare Worker.
    You can pass a name and template to this command optionally. 

    ```
    wrangler generate <name> <template>
    ```

    It will default to the name `wasm-worker` and the [`rustwasm-worker-template`](https://github.com/cloudflare/rustwasm-worker-template).
  - ### 🦀⚙️ `build`
    Build your project using `wasm-pack`.
  - ### 🔬 `preview`
    Preview your project using the cloudflareworkers.com API.
  - ### ☁️ 🆙 `publish`
    Publish your Worker and WebAssembly to Cloudflare.

## Configuration

There are two types of configuration that `wrangler` uses: global user and per project.

- ### Global User

    In Cloudflare's system, you have a User that can have multiple Accounts and Zones. As a result, your User
    is configured globally on your machine. Your Account(s) and Zone(s) will be configured per project, but
    will use your User credentials to authenticate all API calls. This config file is created in a `.wrangler`
    directory in your computer's home directory.

    To set up `wrangler` to work with your Cloudflare user, use the following commands:

    - 🔧 `config`: an interactive command that asks you to pass your `email` and `api` key. Alternatively, 
       you can use the flags `--email` and `--api-key` to the command to skip the interactive part.
    - 🕵️‍♀️ `whoami`: run this command to confirm that your configuration is appropriately set up.
       When successful, this command will print out your user information, including the type of plan you
       are currently on.


- ### Per Project

    Your project will need to have several things configured before you can publish your worker. These values
    are stored in a `wrangler.toml` file that `wrangler generate` will make for you. You will need to manually
    edit this file to add these values before you can publish.

    - `name`: This is the name of your project. It will be the name of your script.
    - `zone_id`: This is the ID of the "zone" or domain you want to run your script on.
    - `account_id`: This is the ID of the account associated with your zone. You might have more than one account,
        so make sure to use the ID of the account associated with the `zone_id` you provide.
    - `route`: This is the route you'd like to use your worker on. You need to include the hostname. Examples:
        - `*example.com/*`
        - `http://example.com/hello`
        - `https://example.com/*/world`

    Cloudflare templates automatically add the `wrangler.toml` file to `.gitignore`.
    
    ⚠️ NEVER PUBLISH CREDENTIALS TO VERSION CONTROL! ⚠️

## ⚓ Installation

1. Install `cargo`:

    Wrangler is installed through [Cargo](https://github.com/rust-lang/cargo#compiling-from-source), a Rust package manager. Rustup, a tool for installing Rust, will also install Cargo. On Linux and macOS systems, `rustup` can be installed as follows:

    ```
    curl https://sh.rustup.rs -sSf | sh
    ```

    Additional installation methods are available [here](https://forge.rust-lang.org/other-installation-methods.html).

1. Install `wrangler`:

    ```
    cargo install wrangler
    ```

1. Troubleshooting OpenSSL errors

    If you are on a Mac, you might encounter an OpenSSL error when attempting to generate a project. You can resolve that issue by installing OpenSSL v1.1 through Homebrew (need to install Homebrew? Instructions available [here](https://brew.sh/)).

    ```
    $ brew install openssl@1.1
    ```

## ⚡ Quick Start

1. Generate a new project:

    ```
    wrangler generate
    ```

1. Move into the new project directory:

    ```
    cd wasm-worker
    ```

1. Build your project:

    ```
    wrangler build
    ```

1. Preview your project:

    ```
    wrangler preview
    ```

1. (optional) Configure with your Cloudflare account:

    ```
    wrangler config <email> <api_key>
    ```

    Configuring your account is required to use the `publish` step, which will push your Worker live to the
    Cloudflare edge. If you don't configure, you can still use `wrangler` to generate, build, and preview
    a Worker.

1. Check your configuration:

    ```
    wrangler whoami
    ```

1. Publish your project:

    ```
    wrangler publish <zone_id>
    ```

    ... where `<zone_id>` is replaced with the `id` for the Cloudflare zone you are publishing to!
