# Changelog

## 🎁 1.3.0

- ### Features

  - **Environments - [EverlastingBugstopper], [issue/385][pull/386]**

    Wrangler 1.3.0 includes supports for **environments**, allowing developers to deploy Workers projects to multiple places. For instance, an application can be deployed to a production URL _and_ a staging URL, without having to juggle multiple configuration files.

    To use environments, you can now pass in `[env.$env_name]` properties in your `wrangler.toml`. Here's an example:

    ```toml
    type = "webpack"
    name = "my-worker-dev"
    account_id = "12345678901234567890"
    zone_id = "09876543210987654321"
    workers_dot_dev = false

    [env.staging]
    name = "my-worker-staging"
    route = "staging.example.com/*"

    [env.production]
    name = "my-worker"
    route = "example.com/*"
    ```

    With multiple environments defined, `wrangler build`, `wrangler preview`, and `wrangler publish` now accept a `--env` flag to indicate what environment you'd like to use, for instance, `wrangler publish --env production`.

    To support developers transitioning to environments, we've written documentation for the feature, including further information about deprecations and advanced usage. [Check out the documentation here!](https://github.com/cloudflare/wrangler/blob/master/docs/environments.md)

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [issue/385]: https://github.com/cloudflare/wrangler/issues/385
    [pull/386]: https://github.com/cloudflare/wrangler/pull/386

  - **KV commands - [ashleymichal], [gabbifish], [issue/339][pull/405]**

    Wrangler 1.3.0 includes commands for managing and updating [Workers KV](https://www.cloudflare.com/products/workers-kv/) namespaces, keys, and values directly from the CLI.

    - `wrangler kv:namespace`

      `wrangler kv:namespace` allows developers to `create`, `list`, and `delete` KV namespaces, from the CLI. This allows Wrangler users to configure new namespaces without having to navigate into the Cloudflare Web UI to manage namespaces. Once a namespace has been created, Wrangler will even give you the exact configuration to copy into your `wrangler.toml` to begin using your new namespace in your project. Neat!

      ```console
      $ wrangler kv:namespace create "MY_KV"
      🌀  Creating namespace with title "worker-MY_KV"
      ✨  Success: WorkersKvNamespace {
        id: "e29b263ab50e42ce9b637fa8370175e8",
        title: "worker-MY_KV",
      }
      ✨  Add the following to your wrangler.toml:
      kv-namespaces = [
        { binding = "MY_KV", id = "e29b263ab50e42ce9b637fa8370175e8" }
      ]
      ```

    - `wrangler kv:key`

      `wrangler kv:key` gives CLI users access to reading, listing, and updating KV keys inside of a namespace. For instance, given a namespace with the binding `KV`, you can directly set keys and values from the CLI, including passing expiration data, such as below:

      ```console
      $ wrangler kv:key put --binding=KV "key" "value" --ttl=10000
      ✨  Success
      ```

    - `wrangler kv:bulk`

      `wrangler kv:bulk` can be used to quickly upload or remove a large number of values from Workers KV, by accepting a JSON file containing KV data. Let's define a JSON file, `data.json`:

      ```json
      [
        {
          "key": "test_key",
          "value": "test_value",
          "expiration_ttl": 3600
        },
        {
          "key": "test_key2",
          "value": "test_value2",
          "expiration_ttl": 3600
        }
      ]
      ```

      By calling `wrangler kv:bulk put --binding=KV data.json`, I can quickly create two new keys in Workers KV - `test_key` and `test_key2`, with the corresponding values `test_value` and `test_value2`:

      ```console
      $ wrangler kv:bulk put --binding=KV data.json
      ✨  Success
      ```

    The KV subcommands in Wrangler 1.3.0 make it super easy to comfortably query and manage your Workers KV data without ever having to leave the command-line. For more information on the available commands and their usage, see [the documentation](https://github.com/cloudflare/wrangler/blob/feat-kv-commands/docs/content/kv_commands.md). 🤯

    [ashleymichal]: https://github.com/ashleymichal
    [gabbifish]: https://github.com/gabbifish
    [issue/339]: https://github.com/cloudflare/wrangler/issues/339
    [pull/405]: https://github.com/cloudflare/wrangler/pull/405

  - **Reduce output from publish command - [EverlastingBugstopper], [issue/523][pull/584]**

    This PR improves the messaging of `wrangler publish`.

    **Before:**

    ```console
    ✨  Built successfully, built project size is 517 bytes.
    ✨  Successfully published your script.
    ✨  Success! Your worker was successfully published. You can view it at example.com/*
    ```

    **After:**

    ```console
    $ wrangler publish
    ✨  Built successfully, built project size is 523 bytes.
    ✨  Successfully published your script to example.com/*
    ```

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [issue/523]: https://github.com/cloudflare/wrangler/issues/523
    [pull/584]: https://github.com/cloudflare/wrangler/pull/584

  - **feat #323: Allow fn & promise from webpack config - [third774], [issue/323][pull/325]**

    This PR fixes Wrangler's handling of `webpack.config.js` files to support functions and promises, as per the [Webpack documentation](https://webpack.js.org/configuration/configuration-types/).

    [third774]: https://github.com/third774
    [issue/323]: https://github.com/cloudflare/wrangler/issues/323
    [pull/325]: https://github.com/cloudflare/wrangler/pull/325

  - **Use webworker target - [xtuc], [issue/477][pull/490]**

    This PR updates how Wrangler builds JavaScript projects with Webpack to conform to the [`webworker`](https://webpack.js.org/configuration/target/) build target. This ensures that projects built with Wrangler are less likely to generate code that isn't supported by the Workers runtime.

    [xtuc]: https://github.com/xtuc
    [issue/477]: https://github.com/cloudflare/wrangler/issues/477
    [pull/490]: https://github.com/cloudflare/wrangler/pull/490

- ### Fixes

  - **Have live reload preview watch over entire rust worker directory - [gabbifish], [pull/535]**

    This change updates the live reload functionality to watch over the entire Rust worker directory, and does not look for `package.json` files that do not typically exist for Rust and WASM projects.

    [gabbifish]: https://github.com/gabbifish
    [pull/535]: https://github.com/cloudflare/wrangler/pull/535

  - **Fix javascript live preview for linux - [gabbifish], [issue/517][pull/528]**

    This PR fixes an issue with Wrangler's live preview (`wrangler preview --watch`) functionality on Linux. In addition, it simplifies the console output for a live preview instance, which could get pretty noisy during development!

    [gabbifish]: https://github.com/gabbifish
    [issue/517]: https://github.com/cloudflare/wrangler/issues/517
    [pull/528]: https://github.com/cloudflare/wrangler/pull/528

- ### Maintenance

  - **Add keywords for npm SEO - [EverlastingBugstopper], [pull/583]**

    This PR improves the discoverability for wrangler on npm by adding keywords to the installer's `package.json`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/583]: https://github.com/cloudflare/wrangler/pull/583

  - **Clean up emoji - [xortive], [pull/455]**

    This PR removes some extraneous unicode that keeps our emoji from displaying correctly in certain terminal emulators, especially for multi-codepoint emoji.

    [xortive]: https://github.com/xortive
    [pull/455]: https://github.com/cloudflare/wrangler/pull/455

- ### Documentation

  - **Add documentation for init to the README - [EverlastingBugstopper], [pull/585]**

    This PR adds documentation in our README for `wrangler init`, which allows you to begin using an existing project with Wrangler.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/585]: https://github.com/cloudflare/wrangler/pull/585

  - **Remove link to docs for installation because they link back to wrangler README - [EverlastingBugstopper], [pull/494]**

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/494]: https://github.com/cloudflare/wrangler/pull/494

  - **Minor formatting fix in README.md - [kentonv], [pull/515]**

    This PR fixes a small syntax issue in the Wrangler README, causing Markdown lists to render incorrectly.

    [kentonv]: https://github.com/kentonv
    [pull/515]: https://github.com/cloudflare/wrangler/pull/515

  - **Fix link to Cloudflare Workers preview service - [dentarg], [pull/472]**

    This PR fixes a link to the [Cloudflare Workers preview service](https://cloudflareworkers.com) in the Wrangler README.

    [dentarg]: https://github.com/dentarg
    [pull/472]: https://github.com/cloudflare/wrangler/pull/472

## 💆🏻‍♂️ 1.2.0

- ### Features

  - **Implement live previewing for wrangler - [xortive], [pull/451]**

    The `wrangler preview` command now supports live previewing! As you develop your project, you can start up a link between your local codebase and the preview service by running `wrangler preview --watch`. Any updates to your project will be passed to the preview service, allowing you to instantly see any changes to your project. This is a _massive_ improvement to the development process for building applications with Workers, and we're super excited to ship it!

    A huge shout-out to @xortive, who almost single-handedly built this feature during his summer internship at Cloudflare, over the last few months. Amazing work! 😍

    [xortive]: https://github.com/xortive
    [pull/451]: https://github.com/cloudflare/wrangler/pull/451

  - **Authenticate calls to preview service when possible - [ashleymichal], [issue/423][issue/431] [pull/429]**

    This PR allows developers to use the preview service with account and user-specific functionality, such as KV support inside of the preview service. Previously, attempting to preview a Workers project with KV bindings would cause an error - this PR fixes that, by allowing `wrangler preview` to make authenticated calls to the preview service.

    [ashleymichal]: https://github.com/ashleymichal
    [issue/423]: https://github.com/cloudflare/wrangler/issues/423
    [issue/431]: https://github.com/cloudflare/wrangler/issues/431
    [pull/429]: https://github.com/cloudflare/wrangler/pull/429

- ### Documentation

  - **Cleanup README and link to workers docs - [EverlastingBugstopper], [pull/440]**

    This PR cleans up the README and adds additional links to the [Workers documentation](https://workers.cloudflare.com/docs) to improve consistency around Wrangler documentation.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/440]: https://github.com/cloudflare/wrangler/pull/440

  - **Link to docs for update instructions - [ashleymichal], [pull/422]**

    We've migrated the "Updating `wrangler`" section of the README to the [Workers documentation](https://workers.cloudflare.com/docs/quickstart/updating-the-cli).

    [ashleymichal]: https://github.com/ashleymichal
    [pull/422]: https://github.com/cloudflare/wrangler/pull/422

- ### Fixes

  - **Fix ignoring exit code from spawnSync - run-wrangler.js - [defjosiah], [issue/433][issue/335] [pull/432]**

    This PR fixes an issue where an NPM-installed `wrangler` would _always_ return an exit code of 0, even when `wrangler` had errored. This improves `wrangler`'s ability to be scripted, e.g. in CI projects.

    [defjosiah]: https://github.com/defjosiah
    [issue/433]: https://github.com/cloudflare/wrangler/issues/433
    [issue/335]: https://github.com/cloudflare/wrangler/issues/335
    [pull/432]: https://github.com/cloudflare/wrangler/pull/432

- ### Maintenance

  - **Test maintenance - [EverlastingBugstopper], [pull/563]**

    This PR cleans up some incorrectly named tests and adds fixtures to support testing new functionality in 1.3.0, such as environments. ✨

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/563]: https://github.com/cloudflare/wrangler/pull/563

  - **Guard test against potential races - [xtuc], [pull/567]**

    This PR fixes a race condition during `wrangler build` that can occur when two builds are taking place at the same time. To fix it, a file lock has been implemented, which Wrangler will wait on until the previous build completes.

    [xtuc]: https://github.com/xtuc
    [pull/567]: https://github.com/cloudflare/wrangler/pull/567

  - **Deny clippy warnings in CI, run rustfmt in --check mode - [xortive], [issue/426][pull/435]**

    This PR updates some of the CI steps for testing Wrangler, to make sure that [rust-clippy](https://github.com/rust-lang/rust-clippy) warnings cause CI to fail. This helps Wrangler code stay well-written and consistent - hooray!

    [xortive]: https://github.com/xortive
    [issue/426]: https://github.com/cloudflare/wrangler/issues/426
    [pull/435]: https://github.com/cloudflare/wrangler/pull/435

  - **Nits: fix clippy warnings - [xortive], [pull/427]**

    [xortive]: https://github.com/xortive
    [pull/427]: https://github.com/cloudflare/wrangler/pull/427

  - **Add repository link to Cargo.toml - [56quarters], [pull/425]**

    [56quarters]: https://github.com/56quarters
    [pull/425]: https://github.com/cloudflare/wrangler/pull/425

## 🏄‍♀️ 1.1.1

- ### Features

  - **Install current version, not latest - [ashleygwilliams], [issue/418][pull/419]**

    Previously the NPM installer for wrangler would always pull the most recent release from Github releases, and the installer did not increase version numbers when Wrangler did. Many users found this confusing. Now the installer will increment versions along with Wrangler releases, and point at specific versions rather than the most recent one at the time of installation.

    [ashleygwilliams]: https://github.com/ashleygwilliams
    [issue/418]: https://github.com/cloudflare/wrangler/issues/418
    [pull/419]: https://github.com/cloudflare/wrangler/pull/419

  - **Improve JSON errors debuggability - [xtuc], [pull/394]**

    This PR improves JSON error output in `wrangler`. Specifically:

    - If a `package.json` file fails to decode, `wrangler` now emits a clearer error:

      ```
      $ wrangler build
      ⬇️  Installing wranglerjs...
      ⬇️  Installing wasm-pack...
      thread 'main' panicked at 'could not parse "./package.json": Error("expected `,` or `}`", line: 4, column: 3)', src/libcore/result.rs:999:5
      ```

    - If the `wranglerjs` backend returns invalid JSON, it now preserves the output file for further investigation. Note that the console doesn't print the output file location by default, and you will need to pass `RUST_LOG=info` while running `wrangler build`, and search for the `--output-file=FILE` argument passed to `wranglerjs`:

      ```
      $ RUST_LOG=info wrangler build
      ⬇️ Installing wasm-pack...
      [2019-08-09T19:28:48Z INFO  wrangler::commands::build::wranglerjs] Running "/Users/kristian/.nvm/versions/node/v12.1.0/bin/node" "/Users/kristian/src/workers/wrangler/wranglerjs" "--output-file=/var/folders/5x/yzqyqst11n518yl8xl7yv1f80000gp/T/.wranglerjs_output5eREv" # ...
      ```

    - If the preview service returns invalid JSON, it now emits a clearer error, and the full output can be seen by using `RUST_LOG=info`.

      Previously:

      ```
      $ wrangler preview
      ⬇️ Installing wasm-pack...
      ⬇️ Installing wranglerjs...
      ✨   Built successfully.
      Error: Error("expected value", line: 2, column: 1)
      ```

      Now:

      ```
      $ wrangler preview
      ⬇️ Installing wranglerjs...
      ⬇️ Installing wasm-pack...
      ✨   Built successfully, built project size is 1 MiB. ⚠️  Your built project has grown past the 1MiB size limit and may fail to deploy. ⚠️  ✨
      Error: https://cloudflareworkers.com/script: Server Error: 502 Bad Gateway
      ```

    [xtuc]: https://github.com/xtuc
    [pull/394]: https://github.com/cloudflare/wrangler/pull/394

- ### Fixes

  - **Fix `wrangler config` for systems with non-unix EOL - [xortive], [issue/389][pull/399]**

    `wrangler config` was not properly truncating whitespace from the end of user input, resulting in a panic when trying to use `wrangler publish`, because `wrangler` would try to create an HTTP header with invalid characters. Now, `wrangler` will properly truncate extra whitespace (including `\r`) from the end of the input into `wrangler config`.

    [xortive]: https://github.com/xortive
    [issue/389]: https://github.com/cloudflare/wrangler/issues/389
    [pull/399]: https://github.com/cloudflare/wrangler/pull/399

- ### Maintenance

  - **Migrate straggler emojis to terminal::emoji - [EverlastingBugstopper], [pull/382]**

    This PR updates the last remaining instances where `wrangler` was using hard-coded emojis for messages, rather than using `terminal::emoji`. In addition, there are two instances where this PR changes the usage of the ⛔ emoji to ⚠️.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/382]: https://github.com/cloudflare/wrangler/pull/382

  - **Move test fixtures to their own directory - [EverlastingBugstopper], [pull/383]**

    This PR aggregates fixtures used in integration tests into a `fixtures` directory to
    make it easier to find/use them.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/383]: https://github.com/cloudflare/wrangler/pull/383

  - **Update issue templates to fit Github's data model - [EverlastingBugstopper], [pull/387]**

    Our previous issue templates were not picked up by Github's user interface. This PR updates the templates to fit the accepted data model, and adds some style tweaks to make the templates easier to use.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/387]: https://github.com/cloudflare/wrangler/pull/387

  - **Move Emoji formatting/messaging into new functions - [ashleymichal], [pull/391]**

    This PR makes improvements to the internal messaging logic of Wrangler, allowing us to be more flexible in how we display information to users.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/391]: https://github.com/cloudflare/wrangler/pull/391

- ### Documentation

  - **Update README to include config, env var changes - [ashleymichal], [pull/379]**

    In 1.1.0 we changed the `config` command to be interactive. This PR updates the README to reflect that change.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/379]: https://github.com/cloudflare/wrangler/pull/379

  - **Add section to readme about Vendored OpenSSL - [xortive], [pull/407]**

    Wrangler has some external OpenSSL dependencies when installing on Linux -- this PR documents those dependencies, and how to install Wrangler using a vendored OpenSSL feature flag:

    ```
    cargo install wrangler --features vendored-openssl
    ```

    [xortive]: https://github.com/xortive
    [pull/407]: https://github.com/cloudflare/wrangler/pull/407

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
