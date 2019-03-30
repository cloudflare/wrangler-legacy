# 🤠 wrangler

[![crates.io](https://meritbadge.herokuapp.com/wrangler)](https://crates.io/crates/wrangler)
[![Build Status](https://dev.azure.com/ashleygwilliams/wrangler/_apis/build/status/cloudflare.wrangler?branchName=master)](https://dev.azure.com/ashleygwilliams/wrangler/_build/latest?definitionId=1&branchName=master)

✨ CHECK OUT THE [TUTORIAL](https://developers.cloudflare.com/workers/webassembly/tutorial/) ✨

`wrangler` is a CLI tool designed for folks who are interested in using Rust-generated WebAssembly on
Cloudflare Workers. This tool gives you the follow commands:

  - 👯 `generate`: scaffold  a `hello-wasm-worker` project, including boilerplate for a Rust library and a
     Cloudflare Worker
  - 🦀⚙️ `build`: build your project using `wasm-pack`
  - 🔬 `preview`: preview your project using the cloudflareworkers.com API
  - ☁️ 🆙 `publish`: publish your Worker and WebAssembly to Cloudflare

To set up `wrangler` to work with your Cloudflare account, use the following commands:

  - `config`: an interactive command that asks you to pass your `email` and `api` key. Alternatively, you
    can use the flags `--email` and `--api-key` to the command to skip the interactive part.
  - 🕵️‍♀️ `whoami`: run this command to confirm that your configuration is appropriately set up. When successful,
    this command will print out your account information, including the type of plan you are currently on.

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
