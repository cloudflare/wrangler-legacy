# Changelog

## 🔑 1.1.0

Wrangler 1.1.0 includes a number of improvements to documentation and project stability, including:

- ### Security

  - **Change global config perm - [xtuc], [pull/286]**

    This PR improves the security of Wrangler's global config file, by restricting access to Wrangler's global user configuration file (`~/.wrangler/config/default.toml`) to the user who ran the wrangler config command. **For existing Wrangler users, please run `wrangler config` again on your machine!** This will fix the permissions of your global config file to be scoped to your user account.

    [xtuc]: https://github.com/xtuc
    [pull/286]: https://github.com/cloudflare/wrangler/pull/286#event-2516759398

  - **Use stdin instead of arguments for wrangler config - [xtuc], [pull/329]**

    We've made the `wrangler config` command interactive – the previous version of the command, `wrangler config $email $apiKey`, would be captured by your terminal's history, often exposing that information in a `~/.bash_history` or a similar file. The new version of `wrangler config` will prompt you for your `email` and `api_key` via `stdin`.

    In addition, this PR also adds support for a `WRANGLER_HOME` environment variable, which will be the location for Wrangler's "home" directory, if you need to customize where Wrangler saves its configuration information.

    [xtuc]: https://github.com/xtuc
    [pull/329]: https://github.com/cloudflare/wrangler/pull/239

* ### Features

  - **Support KV Namespace Configuration - [ashleymichal], [pull/334], add check + error message for pre 1.1.0 kv namespace format - [xortive], [pull/369]**

    Wrangler now supports using [Workers KV][kv] namespaces in your project! To start using KV with your projects, create a namespace in the Cloduflare Dashboard, and the namespace information to your `wrangler.toml` configuration file. The `kv-namespaces` key requires setting a `binding` (the representation of your namespace in your code) and `id`, the namespace ID:

    ```toml
    # wrangler.toml

    [[kv-namespaces]]
    binding = "TODOS"
    id = "0f2ac74b498b48028cb68387c421e279"
    ```

    **If you were previously using the undocumented `kv-namespaces` support in your project config, you'll need to make a few changes to your project to use Wrangler 1.1.0!** KV namespaces now need to be created manually, in the Cloudflare Dashboard, to be able to use them in your Wrangler configuration - previous versions of Wrangler created the namespace for you, but the process is now manual, to allow developers to be more explicit about how their KV namespaces are created.

    For users of the previously undocumented `kv-namespaces` functionality in Wrangler, we've provided a warning and upgrade path, to help you upgrade your KV configuration in your project to the correct format:

    ```
    ⚠️  As of 1.1.0 the kv-namespaces format has been stabilized ⚠️
    💁‍  Please add a section like this in your `wrangler.toml` for each KV Namespace you wish to bind: 💁‍

    [[kv-namespaces]]
    binding = "BINDING_NAME"
    id = "0f2ac74b498b48028cb68387c421e279"

    # binding is the variable name you wish to bind the namespace to in your script.
    # id is the namespace_id assigned to your kv namespace upon creation. e.g. (per namespace)

    Error: ⚠️  Your project config has an error ⚠️
    ```

    [xortive]: https://github.com/xortive
    [pull/369]: https://github.com/cloudflare/wrangler/pull/369

    This is the initial part of a lot of incredible work being done on supporting Workers KV in Wrangler. If you're interested in what's up next, check out our [next milestone](https://github.com/cloudflare/wrangler/milestone/7).

    [ashleymichal]: https://github.com/ashleymichal
    [pull/334]: https://github.com/cloudflare/wrangler/pull/334

  - **Configure Workers KV in your wrangler.toml - [ashleymichal], [pull/333]**

    If you've tried to use [Workers KV][kv] in Wrangler, you've probably had a bad time! This PR, along with [#334](https://github.com/cloudflare/wrangler/pull/334), build support for handling and correctly uploading KV namespace information with your Wrangler project:

    ```
    [[kv-namespaces]]
    binding = "NAMESPACE"
    id = "0f2ac74b498b48028cb68387c421e279"
    ```

    [ashleymichal]: https://github.com/ashleymichal
    [pull/333]: https://github.com/cloudflare/wrangler/pull/333
    [kv]: https://www.cloudflare.com/products/workers-kv/

  - **Use ENV variables to configure Wrangler - [AaronO], [pull/225]**

    Previously, Wrangler required a global configuration file to be able to run. As many users may use Wrangler in situations where they don't have an interactive terminal, meaning they can't instantiate a config file using `wrangler config`, this PR allows Wrangler to run even if the config file doesn't exist. This change means that users can also configure Wrangler exclusively with environment variables, using `$CF_API_KEY` for your Cloudflare API key, and `$CF_EMAIL` for your Cloudflare account email.

    [aarono]: https://github.com/AaronO
    [pull/225]: https://github.com/cloudflare/wrangler/pull/225

  - **Adds more descriptive subdomain errors - [EverlastingBugstopper], [issue/207][pull/259]**

    It's super easy to grab a workers.dev subdomain using the `subdomain` command in `wrangler` – so easy, in fact, that many people were trying to use it without even having a Cloudflare account! `wrangler` now warns users when they attempt to add a subdomain without configuring their `account_id` in `wrangler.toml`, as well as when you've already registered a subdomain, or if the subdomain you're trying to register has already been claimed.

    [everlastingbugstopper]: https://github.com/EverlastingBugstopper
    [issue/207]: https://github.com/cloudflare/wrangler/issue/207
    [pull/259]: https://github.com/cloudflare/wrangler/pull/259

  - **Allow custom webpack configuration in wrangler.toml - [EverlastingBugstopper], [issue/246][pull/253]**

    If you'd like to bring your own Webpack config to your Workers project, you can now specify a `webpack_config` key in `wrangler.toml`:

    ```toml
    webpack_config: webpack.prod.js
    ```

    [everlastingbugstopper]: https://github.com/EverlastingBugstopper
    [issue/246]: https://github.com/cloudflare/wrangler/issue/246
    [pull/253]: https://github.com/cloudflare/wrangler/pull/253

  - **Add issue templates for bug reports and feature requests - [gabbifish], [issue/250][pull/350]**

    To make it easier for us to diagnose problems and support user feedback, we've added issue templates to make it easier for users to submit bug reports and feature requests.

    [gabbifish]: https://github.com/gabbifish
    [issue/250]: https://github.com/cloudflare/wrangler/issue/250
    [pull/350]: https://github.com/cloudflare/wrangler/pull/350

  - **Display commands in their defined order - [Electroid], [pull/236]**

    We've re-arranged the order of the commands when you run `wrangler` without any subcommands, so that commonly-used commands are easier to find!

    [electroid]: https://github.com/Electroid
    [pull/233]: https://github.com/cloudflare/wrangler/pull/236

  - **Show project size on build - [xtuc], [pull/205]**

    Once the build is finished, `wrangler` now prints the compressed size of the script, and, if available, the Wasm binary size:

    ```
    $ wrangler publish
    ⬇️ Installing wranglerjs...
    ⬇️ Installing wasm-pack...
    ✨   Built successfully, built project size is 517 bytes. ✨
    ```

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

  - **Remove OpenSSL installation step from README - [xortive], [issue/355][pull/356]**

    Due to an OpenSSL dependency in [`cargo-generate`](https://github.com/ashleygwilliams/cargo-generate/), Wrangler required developers to install OpenSSL as part of the setup process for using Wrangler. [Version 0.3.1](https://github.com/ashleygwilliams/cargo-generate/pull/170) of `cargo-generate` has removed this requirement, and you no longer need to install OpenSSL manually to use Wrangler. Hooray!

    [xortive]: https://github.com/xortive
    [issue/355]: https://github.com/cloudflare/wrangler/issues/355
    [pull/356]: https://github.com/cloudflare/wrangler/pull/356

  - **Fix issue previewing a project with KV namespaces - [ashleymichal], [pull/353]**

    This PR fixes a critical bug where using `wrangler preview` on a project with [Workers KV][kv] namespaces causes the command to throw an error.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/353]: https://github.com/cloudflare/wrangler/pull/353

  - **Enforce one Webpack entry in configuration - [xtuc], [pull/245]**

    `wrangler` now returns an error during the build process if you use a webpack configuration with more than one export – `wrangler` needs to have a single known export from webpack to know what to build!

    [xtuc]: https://github.com/xtuc
    [pull/245]: https://github.com/cloudflare/wrangler/pull/245

  - **Update default template for Rust project type - [EverlastingBugstopper], [pull/309]**

    Previously, when passing `--type rust` to `wrangler generate`, the only indication that it worked was that the type in `wrangler.toml` was `rust`. There were no Rust files in the default template, for a Rust-type project. Now, when running `wrangler generate --type rust`, `wrangler` will use [rustwasm-worker-template](https://github.com/cloudflare/rustwasm-worker-template) when generating a new project.

    [pull/309]: https://github.com/cloudflare/wrangler/pull/309

  - **Stop cleaning webpack build artifacts - [EverlastingBugstopper], [pull/307]**

    The configuration for Webpack projects in Wrangler was over-eager to clean build artifacts, which in the worst-case, caused Wrangler to remove source code for developers' projects - oops! This fix relaxes the Webpack cleaning process, to ensure that building a Wrangler project is a bit safer.

    [pull/307]: https://github.com/cloudflare/wrangler/pull/307

  - **Correct binding format - [xtuc], [pull/260]**

    Previously, `wrangler` was incorrectly sending up a `binding` object to the Cloudflare API, whenever we attempted to update a script's bindings. This fix renames it to `bindings`, and uses an array, as per the Cloudflare API requirements.

    [xtuc]: https://github.com/xtuc
    [pull/260]: https://github.com/cloudflare/wrangler/pull/260

  - **Correctly pass Wasm module - [xtuc], [pull/261]**

    To ensure that a wasm module is successfully passed between `wranglerjs` and `wrangler`, the wasm module is now encoded and decoded from base64, avoiding any potential loss of data.

    [xtuc]: https://github.com/xtuc
    [pull/261]: https://github.com/cloudflare/wrangler/pull/261

  - **Check for `account_id` and `zone_id` before publishing - [xtuc], [issue/170][pull/192]**

    The `publish` subcommand in `wrangler` now ensures that you have an `account_id` and `zone_id` configured in your `wrangler.toml` file before attempting to publish, instead of failing during the publishing process.

    [xtuc]: https://github.com/xtuc
    [issue/170]: https://github.com/cloudflare/wrangler/issues/170
    [pull/192]: https://github.com/cloudflare/wrangler/pull/192

  - **Fix Rust ref issue with `wranglerjs` builds - [xtuc], [pull/227]**

    When `wranglerjs` built a project, it incorrectly referred to the output of that build process without using a Rust reference - this PR fixes that issue and allows `wranglerjs` to correctly take your bundle, and your project's metadata, and put it all together in a nice little package to send up to the Cloudflare API. Hooray, working projects!

    [xtuc]: https://github.com/xtuc
    [pull/227]: https://github.com/cloudflare/wrangler/pull/227

* ### 📖 Documentation

  - **feat(docs): add CONTRIBUTING.md - [ashleygwilliams], [pull/268]**

    We've created a shiny new [contribution guide](https://github.com/cloudflare/wrangler/blob/master/CONTRIBUTING.md) to help contributors understand how the project works, and how the team triages incoming issues and feature requests. This should make it easier for us to work through your feedback on Wrangler, as well as give you some insight into how we work. Woo-hoo! 🎉

    [ashleygwilliams]: https://github.com/ashleygwilliams
    [pull/268]: https://github.com/cloudflare/wrangler/pull/268#event-2516878124

  - **Update README to include KV config info - [ashleymichal], [pull/319]**

    You can now create [Workers KV][kv] namespaces from inside of your `wrangler.toml` configuration file - this has been documented in the README.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/319]: https://github.com/cloudflare/wrangler/pull/319

  - **Clarified intro link in README - [tomByrer], [pull/257]**

    [tomByrer]: https://github.com/tomByrer
    [pull/257]: https://github.com/cloudflare/wrangler/pull/257

  - **Make it more clear that you can install Wrangler though npm - [zackbloom], [pull/241]**

    [zackbloom]: https://github.com/zackbloom
    [pull/241]: https://github.com/cloudflare/wrangler/pull/241

  - **Document (lightly) the Wrangler 1.0.0 release - [signalnerve], [pull/204]**

    [signalnerve]: https://github.com/signalnerve
    [pull/204]: https://github.com/cloudflare/wrangler/pull/204

  - **Add README for Wrangler.js package - [ashleygwilliams], [pull/196]**

    [ashleygwilliams]: https://github.com/ashleygwilliams
    [pull/196]: https://github.com/cloudflare/wrangler/pull/196

* ### 🔧 Maintenance

  - **Better error printing - [xortive], [pull/327]**

    We've updated how we log errors in Wrangler's output to make it a little easier to read. If you're a Rust developer interested in _how_ we did this, check out the [pull request][pull/327]!

    [xortive]: https://github.com/xortive
    [pull/327]: https://github.com/cloudflare/wrangler/pull/327

  - **Always use multipart upload - [ashleymichal], [issue/280][pull/328]**

    We've updated how Wrangler uploads projects to the Workers runtime - as Wrangler supports the entire Workers ecosystem (including tools like [Workers KV][kv], the publishing process should use [multipart forms](https://dev.to/sidthesloth92/understanding-html-form-encoding-url-encoded-and-multipart-forms-3lpa) to allow uploading all of the data that represents a Workers project in a single step.

    [ashleymichal]: https://github.com/ashleymichal
    [issue/280]: https://github.com/cloudflare/wrangler/issue/280
    [pull/328]: https://github.com/cloudflare/wrangler/pull/328

  - **unify upload form building - [ashleymichal], [pull/329]**

    This PR brings consistency to the way that metadata is handled during Wrangler's `preview` and `build` commands: previously, a `metadata.json` file handled the bindings for your Wrangler project, but because it was handled differently for various project types (such as `webpack` or `rust`), it led to inconsistent behavior during the upload process. This PR removes usage of `metadata.json` in favor of building metadata for each project type in-memory, which will then be uploaded directly to Workers platform. This work is foundational for improved Workers KV support in Wrangler 🙌

    [ashleymichal]: https://github.com/ashleymichal
    [pull/329]: https://github.com/cloudflare/wrangler/pull/329

  - **`wrangler preview` integration tests - [EverlastingBugstopper], [pull/363]**

    Wrangler now includes integration tests for `wrangler preview`, testing every project type that Wrangler supports with our [preview service](https://cloudflareworkers.com/).

    [everlastingbugstopper]: https://github.com/EverlastingBugstopper
    [pull/363]: https://github.com/cloudflare/wrangler/pull/363

  - **Add user agent - [xtuc], [issue/234][pull/236]**

    For every outgoing request, `wrangler` includes a `User-Agent` header to clearly indicate to servers and APIs that a `wrangler` client is making a request: `wrangler/dev` in debug mode and `wrangler/$version` in release mode.

    [xtuc]: https://github.com/xtuc
    [issue/234]: https://github.com/cloudflare/wrangler/issue/234
    [pull/236]: https://github.com/cloudflare/wrangler/pull/236

  - **Terminal messaging abstraction - [ashleymical], [issue/219][pull/263]**

    We've made improvements to Wrangler's terminal output functionality, with support for various log levels and implementations in Wrangler's API for easily using the log levels in future development.

    The new terminal output functionality can be used by importing the `terminal::message` crate:

    ```rust
    use crate::terminal::message;

    message::info("Building project") // "💁‍ Building project"
    message::success("Your project has been deployed!") // "✨ Your project has been deployed!"

    // Other available functions:
    // message::warn, message::user_error, message::working, message::preview
    ```

    [ashleymical]: https://github.com/ashleymichal
    [issue/219]: https://github.com/cloudflare/wrangler/issues/219
    [pull/263]: https://github.com/cloudflare/wrangler/pull/263

  - **Remove pre-push hooks - [EverlastingBugstopper], [pull/308]**

    Previous versions of Wrangler included pre-push hooks to ensure that code was linted before being pushed up to Git. This hook made it difficult to manage in-progress work, so the hooks have been removed.

    [pull/308]: https://github.com/cloudflare/wrangler/pull/308

  - **Use serde for metadata - [xtuc], [pull/285]**

    This change adds proper construction of the worker metadata, previously, it was an error-prone string.

    [pull/285]: https://github.com/cloudflare/wrangler/pull/285

  - **Refactor: Conditional per command in main - [ashleymical], [pull/279]**

    The `src/main.rs` file in Wrangler has been rewritten so that the layout of the file is easier to read.

    [ashleymical]: https://github.com/ashleymical
    [pull/279]: https://github.com/cloudflare/wrangler/pull/279

  - **Add an authenticated HTTP client - [Electroid], [issue/238][pull/267]**

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

  - **Reorganize wranglerjs src - [xtuc], [pull/202][issue/154] [issue/155]**

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
