# Changelog

## 🔎 1.1.0

Wrangler 1.1.0 includes a number of improvements to documentation and project stability, including: 

- ### Features

  - **Add message module for terminal output - [ashleymical], [issue/219] [pull/263]**

    We've made improvements to Wrangler's terminal output functionality, with support for various log levels and implementations in Wrangler's API for easily using the log levels in future development.

    The new terminal output functionality can be used by importing the `terminal::message` crate:

    ```rust
    use crate::terminal::message;

    message::info("Building project") // "💁‍ Building project"
    message::success("Your project has been deployed!") // "✨ Your project has been deployed!"

    // Other available functions:
    // message::warn, message::user_error, message::service_error, message::working, message::preview
    ```

    [ashleymical]: https://github.com/ashleymichal
    [issue/219]: https://github.com/cloudflare/wrangler/issues/219
    [pull/263]: https://github.com/cloudflare/wrangler/pull/263

  - **Adds more descriptive subdomain errors - [EverlastingBugstopper], [issue/207] [pull/259]**

    It's super easy to grab a workers.dev subdomain using the `subdomain` command in `wrangler` – so easy, in fact, that many people were trying to use it without even having a Cloudflare account! `wrangler` now warns users when they attempt to add a subdomain without configuring their `account_id` in `wrangler.toml`, as well as when you've already registered a subdomain, or if the subdomain you're trying to register has already been claimed. 

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [issue/207]: https://github.com/cloudflare/wrangler/issue/207
    [pull/259]: https://github.com/cloudflare/wrangler/pull/259

  - **Allow custom webpack configuration in wrangler.toml - [EverlastingBugstopper], [issue/246] [pull/253]**

    If you'd like to bring your own Webpack config to your Workers project, you can now specify a `webpack_config` key in `wrangler.toml`:

    ```toml
    webpack_config: webpack.prod.js
    ```

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [issue/246]: https://github.com/cloudflare/wrangler/issue/246
    [pull/253]: https://github.com/cloudflare/wrangler/pull/253

  - **Enforce one Webpack entry in configuration - [xtuc], [pull/245]**

     `wrangler` now returns an error during the build process if you use a webpack configuration with more than one export – `wrangler` needs to have a single known export from webpack to know what to build!

    [xtuc]: https://github.com/xtuc
    [pull/245]: https://github.com/cloudflare/wrangler/pull/245

  - **Add user agent - [xtuc], [issue/234] [pull/236]**

    For every outgoing request, `wrangler` includes a `User-Agent` header to clearly indicate to servers and APIs that a `wrangler` client is making a request: `wrangler/dev` in debug mode and `wrangler/$version` in release mode.

    [xtuc]: https://github.com/xtuc
    [issue/234]: https://github.com/cloudflare/wrangler/issue/234
    [pull/236]: https://github.com/cloudflare/wrangler/pull/236

  - **Display commands in their defined order - [Electroid], [pull/236]**

    We've re-arranged the order of the commands when you run `wrangler` without any subcommands, so that commonly-used commands are easier to find!

    [Electroid]: https://github.com/Electroid 
    [pull/233]: https://github.com/cloudflare/wrangler/pull/236

  - **Show sizes - [xtuc], [pull/205]**

    Once the build is finished, `wrangler` now prints the compressed size of the script, and, if available, the Wasm binary size.

    [xtuc]: https://github.com/xtuc
    [pull/205]: https://github.com/cloudflare/wrangler/pull/205

  - **Add HTTP prefix to publish command output. - [elithrar], [pull/198]**

    Prefix "https://" in front of the "script available" output to allow shells to automatically detect it as a link. Many shells will allow you to click directly on the URL from inside the terminal (such as iTerm's "CMD-Click"), making it much easier to navigate to your subdomains, or any published Workers applications.

    [elithrar]: https://github.com/elithrar
    [pull/198]: https://github.com/cloudflare/wrangler/pull/198

  - **Build: add message and emoji - [xtuc], [pull/193]**

    The `wrangler` team _really_ loves emoji, so we made sure to send a little bit of ✨ cheer ✨ your way, via a new message and emoji, whenever you use the `build` subcommand. 💌

    [xtuc]: https://github.com/xtuc
    [pull/193]: https://github.com/cloudflare/wrangler/pull/193

- ### 🤕 Fixes

  - **Correct binding format - [xtuc], [pull/260]**

    Previously, `wrangler` was incorrectly sending up a `binding` object to the Cloudflare API, whenever we attempted to update a script's bindings. This fix renames it to `bindings`, and uses an array, as per the Cloudflare API requirements.

    [xtuc]: https://github.com/xtuc
    [pull/260]: https://github.com/cloudflare/wrangler/pull/260

  - **Correctly pass Wasm module - [xtuc], [pull/261]**

    To ensure that a wasm module is successfully passed between `wranglerjs` and `wrangler`, the wasm module is now encoded and decoded from base64, avoiding any potential loss of data.

    [xtuc]: https://github.com/xtuc
    [pull/261]: https://github.com/cloudflare/wrangler/pull/261

  - **Check for `account_id` and `zone_id` before publishing - [xtuc], [issue/170] [pull/192]**

    The `publish` subcommand in `wrangler` now ensures that you have an `account_id` and `zone_id` configured in your `wrangler.toml` file before attempting to publish, instead of failing during the publishing process.

    [xtuc]: https://github.com/xtuc
    [issue/170]: https://github.com/cloudflare/wrangler/issues/170
    [pull/192]: https://github.com/cloudflare/wrangler/pull/192

  - **Pass wranglerjs output as ref - [xtuc], [pull/227]**

    When `wranglerjs` built a project, it incorrectly referred to the output of that build process without using a Rust reference - this PR fixes that issue and allows `wranglerjs` to correctly take your bundle, and your project's metadata, and put it all together in a nice little package to send up to the Cloudflare API. Hooray, working projects!

    [xtuc]: https://github.com/xtuc
    [pull/227]: https://github.com/cloudflare/wrangler/pull/227

  - **Fix installer - [xtuc], [pull/195]**

    Previously, `wranglerjs` would refer to a directory that potentially didn't exist on your machine, depending on your operating system. This change defaults the `wranglerjs` config file to `.wrangler` in your home directory, ensuring consistency throughout operating systems!

    [xtuc]: https://github.com/xtuc
    [pull/195]: https://github.com/cloudflare/wrangler/pull/195


- ### 📖 Documentation

  - **Clarified intro link in README - [tomByrer], [pull/257]**

    [tomByrer]: https://github.com/tomByrer
    [pull/257]: https://github.com/cloudflare/wrangler/pull/257

  - **Make it more clear that you can install Wrangler though npm - [zackbloom], [pull/241]**

    [zackbloom]: https://github.com/zackbloom
    [pull/241]: https://github.com/cloudflare/wrangler/pull/241

  - **Document (lightly) the Wrangler 1.0.0 release - [signalnerve], [pull/204]**

    [signalnerve]: https://github.com/signalnerve
    [pull/204]: https://github.com/cloudflare/wrangler/pull/204

  - **Pkg readme - [ashleygwilliams], [pull/196]**

    [ashleygwilliams]: https://github.com/ashleygwilliams
    [pull/196]: https://github.com/cloudflare/wrangler/pull/196

- ### 🔧 Maintenance

  - **Use serde for metadata - [xtuc], [pull/285]**

    This change adds proper construction of the worker metadata, previously, it was an error-prone string.

    [pull/285]: https://github.com/cloudflare/wrangler/pull/285

  - **Refactor: Conditional per command in main - [ashleymical], [pull/279]**

    [ashleymical]: https://github.com/ashleymical
    [pull/279]: https://github.com/cloudflare/wrangler/pull/279

  - **Add an authenticated HTTP client - [Electroid], [issue/238] [pull/267]**

    All HTTP requests to the Cloudflare API are now made with an authenticated HTTP client.

    [Electroid]: https://github.com/Electroid
    [issue/238]: https://github.com/cloudflare/wrangler/issue/238
    [pull/267]: https://github.com/cloudflare/wrangler/pull/267

  - **Move metadata generation at publish-time - [xtuc], [pull/237]**

    [author]: https://github.com/xtuc
    [pull/237]: https://github.com/cloudflare/wrangler/pull/237

  - **Pin webpack version - [xtuc], [pull/228]**

    Adds a better control over webpack's version, avoiding possible upstream issues.

    [xtuc]: https://github.com/xtuc
    [pull/228]: https://github.com/cloudflare/wrangler/pull/228

  - **Remove empty file - [xtuc], [pull/216]**

    [xtuc]: https://github.com/xtuc
    [pull/216]: https://github.com/cloudflare/wrangler/pull/216

  - **test: improve metadata coverage - [xtuc], [pull/214]**

    [xtuc]: https://github.com/xtuc
    [pull/214]: https://github.com/cloudflare/wrangler/pull/214

  - **Reorganize wranglerjs src - [xtuc], [pull/202] [issue/154] [issue/155]**

    [xtuc]: https://github.com/xtuc
    [issue/154]: https://github.com/cloudflare/wrangler/issue/154
    [issue/155]: https://github.com/cloudflare/wrangler/issue/155
    [pull/202]: https://github.com/cloudflare/wrangler/pull/202

  - **Minor spelling fix - [adaptive], [pull/200]**

    [adaptive]: https://github.com/adaptive
    [pull/200]: https://github.com/cloudflare/wrangler/pull/200


## 👷‍♀️ 1.0.0

Wrangler 1.0.0 has been released! The first major version of Wrangler makes the tool the preferred development and deployment tool for JavaScript and Rust projects to the Cloudflare Workers platform.

This release includes many changes to the developer experience for Wrangler, including:

- Support for installing wrangler via npm: `npm install @cloudflare/wrangler -g`.
- Support for various project types, including `javascript` and `webpack`.
  - The default project type for Wrangler is now `webpack`.
- Support for creating and binding [Workers KV](https://workers.cloudflare.com/docs/reference/storage/overview/) namespaces via your project's configuration file.
- Enhancements to error messages and validation of your project before building and deploying.
- Fixes for Windows usage of Wrangler.
- Fixes for cross-platform console output.

## 💥 0.1.1

- ### 🤕 Fixes

  - **Fix `publish` and `preview` bug for projects with a `-` - [jaysonsantos], [issue/36][pull/38]**

    Rust is a sometimes surprisingly opinionated language! When your `Cargo.toml` specifies a project
    name with a hyphen(`-`) in the name, the Rust compiler will implicitly understand this as a `_` for
    all imports and when it creates compiled artifacts it will name them with a `_`.

    The original implementation of `wrangler` skipped over this, and as a result would go looking for a
    wasm file with a `-` when it should have been looking for a `_`. This resulted in a bit of a gross
    error message that stated that a file was not found.

    We've fixed this now- so go ahead and name your packages with `-`s!

    [jaysonsantos]: https://github.com/jaysonsantos
    [issue/36]: https://github.com/cloudflare/wrangler/issues/36
    [pull/38]: https://github.com/cloudflare/wrangler/pull/38

- ### 📖 Documentation

  - **Install instructions with OpenSSL troubleshooting notes - [AustinCorridor], [issue/35][pull/43]**

    Because of `wrangler`'s use of `cargo-generate` we use OpenSSL. Classically, this is a tricky
    dependency. Some users may run into issue with it! We've documented the steps to fix it on MacOS-
    if you run into this on other platforms, we'd love a PR!

    [AustinCorridor]: https://github.com/AustinCorridor
    [issue/35]: https://github.com/cloudflare/wrangler/issues/35
    [pull/43]: https://github.com/cloudflare/wrangler/pull/43

  - **Typo and casing fixes - [neynah], [pull/42]**

    First releases almost always have some typos. Now they're fixed!

    [neynah]: https://github.com/neynah
    [pull/42]: https://github.com/cloudflare/wrangler/pull/42

## 🌌 0.1.0

- ### 🌊 [Hello World!](https://blog.cloudflare.com/introducing-wrangler-cli/)
