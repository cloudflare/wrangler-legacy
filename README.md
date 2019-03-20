# 🤠 wrangler

[![Build Status](https://travis-ci.com/ashleygwilliams/wrangler.svg?token=hHeDp9pQmz9kvsgRNVHy&branch=master)](https://travis-ci.com/ashleygwilliams/wrangler)

`wrangler` is a CLI tool designed for folks who are interested in using Rust-generated WebAssembly on
Cloudflare Workers. This tool gives you the follow commands:

  - 👯 `generate`: scaffold  a `hello-wasm-worker` project, including boilerplate for a Rust library and a
     Cloudflare worker 
  - 🦀⚙️ `build`: build your project using `wasm-pack`
  - 🔬 `preview`: preview your project using the cloudflareworkers.com API
  - ☁️ 🆙 `publish`: publish your worker and WebAssembly to Cloudflare

To set up `wrangler` to work with your Cloudflare account, use the following commands:

  - `config`: an interactive command that asks you to pass your `email` and `api` key. Alternatively, you
    can use the flags `--email` and `--api-key` to the command to skip the interactive part.
  - 🕵️‍♀️ `whoami`: run this command to confirm that your configuration is approrpriately set up. When successful,
    this command will print out your account information, including the type of plan you are currently on.

    ⚠️ NEVER PUBLISH CREDENTIALS TO VERSION CONTROL! ⚠️

## ⚡ Quick Start

1. Install `wrangler`:

    *for now*...
    - build one from this source
    - ask ashley for a binary

    *eventually*.... 
    - (preferred) install a binary via the [GitHub Release tab]
    - `cargo install wrangler`

2. Configure with you Cloudflare account:

    ```
    wrangler config <email> <api_key>
    ``` 

2. Generate a new project:

    ```
    wrangler generate
    ```

3. Move into the new project directory:
    ```

    cd wasm-worker
    ```

4. Build your project:

    ```
    wrangler build
    ```

5. Preview your project:

    ```
    wrangler preview
    ```

6. Publish your project:

    ```
    wrangler publish <zone_id>
    ```

    ... where `<zone_id>` is replaced with the `id` for the Cloudflare zone your are publishing to!
