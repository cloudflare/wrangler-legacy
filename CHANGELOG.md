# Changelog

## v1.17.0

- ### Features

  - **feat: capture panics and generate error report - [nilslice], [pull/1888]**

    This PR adds support for wrangler error reporting. Currently, all panics are caught and a report is generated, then written to disk. `wrangler report` or `wrangler report --log <file-name.log>` will upload the corresponding error report to Cloudflare

    [nilslice]: https://github.com/nilslice
    [pull/1888]: https://github.com/cloudflare/wrangler/pull/1888

- ### Fixes

  - **fix: clarify error messages around durable objects beta - [nilslice], [pull/1921]**

    Disambiguates error message described in #1859, adds more clarity around other places where Durable Object usage in beta may conflict with `wrangler` functionality.

    [nilslice]: https://github.com/nilslice
    [pull/1921]: https://github.com/cloudflare/wrangler/pull/1921

  - **Fix filtering by extension for js assets - [rubenamorim], [pull/1722]**

    Fixes [#1719](https://github.com/cloudflare/wrangler/issues/1719)

    [rubenamorim]: https://github.com/rubenamorim
    [pull/1722]: https://github.com/cloudflare/wrangler/pull/1722

  - **fix: use latest cloudflare api client, resolving wrangler whoami issue - [nilslice], [pull/1920]**

    Updates `cloudflare-rs` crate to https://github.com/cloudflare/cloudflare-rs/commit/ae936d4b180155bafe5c44482a746fa490513df2, which should fix #1914.

    [nilslice]: https://github.com/nilslice
    [pull/1920]: https://github.com/cloudflare/wrangler/pull/1920

  - **Handle String panic payloads when generating reports - [ObsidianMinor], [pull/1934]**

    Standard panics will only produce &str and String, but we were only handling &str, so this adds handling for String.

    [obsidianminor]: https://github.com/ObsidianMinor
    [pull/1934]: https://github.com/cloudflare/wrangler/pull/1934

- ### Maintenance

  - **chore: tokio ecosystem update - [nataliescottdavidson], [pull/1886]**

    Update [tokio ecosystem](https://github.com/tokio-rs/tokio) crates including hyper, rustls, openssl and necessary API changes
    Rewrite get_tunnel_url to use tokio-retry

    [nataliescottdavidson]: https://github.com/nataliescottdavidson
    [pull/1886]: https://github.com/cloudflare/wrangler/pull/1886

  - **failure --> anyhow - [caass], [pull/1881]**

    Follow up to [#1880](https://github.com/cloudflare/wrangler/pull/1880)

    Switch from deprecated `failure` to `anyhow`. [read this](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md)

    [caass]: https://github.com/caass
    [pull/1881]: https://github.com/cloudflare/wrangler/pull/1881

  - **further reduce complexity of main - [nilslice], [pull/1937]**

    In many ways, thanks to @ObsidianMinor's work on #1932, we can split up the cli code further and keep a simple `main.rs`.

    [nilslice]: https://github.com/nilslice
    [pull/1937]: https://github.com/cloudflare/wrangler/pull/1937

  - **rebases & updates durable-objects-rc branch - [nilslice], [pull/1919]**

    [nilslice]: https://github.com/nilslice
    [pull/1919]: https://github.com/cloudflare/wrangler/pull/1919

  - **Reduce cognitive complexity in main - [ObsidianMinor], [pull/1932]**

    This replaces our massive clap App with a much simpler Cli struct that
    has all the same information in an easier to understand and modify
    format (types).

    [obsidianminor]: https://github.com/ObsidianMinor
    [pull/1932]: https://github.com/cloudflare/wrangler/pull/1932

  - **sorry i got excited with dependabot - [caass], [pull/1917]**

    [caass]: https://github.com/caass
    [pull/1917]: https://github.com/cloudflare/wrangler/pull/1917

  - **update copy on text binding size limit - [caass], [pull/1911]**

    [caass]: https://github.com/caass
    [pull/1911]: https://github.com/cloudflare/wrangler/pull/1911

  - **Updated cloudflared download link - [arunesh90], [pull/1900]**

    The commit included with this PR updates the download link for cloudflared, as the previous link is no longer current and is now a redirect to https://developers.cloudflare.com/cloudflare-one/connections/connect-apps

    [arunesh90]: https://github.com/arunesh90
    [pull/1900]: https://github.com/cloudflare/wrangler/pull/1900

  - **Upgrade to GitHub-native Dependabot - [dependabot], [pull/1887]**

    [dependabot]: https://dependabot.com/
    [pull/1887]: https://github.com/cloudflare/wrangler/pull/1887


## 🍏 v1.16.1

- ### Features
  - **Add `wasm_modules` config field for bundling arbitrary WebAssembly modules. - [losfair], [pull/1803]**

    Currently it seems that wrangler only supports WebAssembly modules included from a `rust` or `webpack` project.

    This pull request enables the inclusion of arbitrary WebAssembly modules through a `wasm_modules` config field.

    [losfair]: https://github.com/losfair
    [pull/1803]: https://github.com/cloudflare/wrangler/pull/1803

- ### Fixes
  - **fix: use x86_64 arch for pre-built downloads on M1 devices - [nilslice], [pull/1876]**

    This PR forces the use of a pre-built x86_64 binary on a aarch64/arm64 Apple system. For M1 devices specifically, it will fix `wrangler generate`, and `wrangler build` for `type = "rust"` wrangler projects.

    There is another semi-related
    ... truncated

    [nilslice]: https://github.com/nilslice
    [pull/1876]: https://github.com/cloudflare/wrangler/pull/1876

  - **chore: include @cloudflare/binary-install directly as source - [nilslice], [pull/1877]**

    This PR inlines the `@cloudflare/binary-install` dependency as a quickfix for #1811, in which usage reports indicated that the module occasionally failed to install.

    I tested this by running `npm i && node run-wrangler.js` on both `npm
    ... truncated

    [nilslice]: https://github.com/nilslice
    [pull/1877]: https://github.com/cloudflare/wrangler/pull/1877

  - **Stop assuming KV values are UTF8 text - [koeninger], [pull/1878]**

    Implementation of kv:key get is calling text() on the result of the api call, so it's replacing non-utf8 values with ef bf bd, the unicode REPLACEMENT CHAR. KV values can be arbitrary bytes, we shouldn't assume they're UTF8 text, so this p
    ... truncated

    [koeninger]: https://github.com/koeninger
    [pull/1878]: https://github.com/cloudflare/wrangler/pull/1878


## 1.16.0

- ### Features

  - **Custom Builds and Modules - [xortive], [caass], [pull/1855]**

    Custom Builds and Modules has finally landed in a main release!
    There's too many new features to write about in a changelog, so here's a
    [link to the docs](https://developers.cloudflare.com/workers/cli-wrangler/configuration#build).

    [xortive]: https://github.com/xortive
    [caass]: https://github.com/caass
    [pull/1855]: https://github.com/cloudflare/wrangler/pull/1855

  - **add `--format` option, including default `json` and new `pretty` - [caass], [pull/1851]**

    You can now pass `--format pretty` to `wrangler tail` to get pretty printed logs!
    `--format json` is also available, and gives you the existing JSON-formatted logs.

    [caass]: https://github.com/caass
    [pull/1851]: https://github.com/cloudflare/wrangler/pull/1851

- ### Fixes

  - **Revert "Print line and column numbers for exception thrown (#1645)" - [Electroid], [pull/1835]**

    This reverts commit 74a89f7c383bc22758cbe55096ce3016c5c319d7.

    Closes #1826

    This commit is causing `wrangler dev` to not show uncaught exceptions. Reverting `chrome-devtools-rs` was also necessary.

    We have a fix in progress to fix the underlying issue and re-introduce line and column numbers.

    [electroid]: https://github.com/Electroid
    [pull/1835]: https://github.com/cloudflare/wrangler/pull/1835

  - **Don't generate `usage_model = ""` by default - [xortive], [issues/1850]**

    Generating `usage_model = ""` by default was violating the toml spec, which broke
    `wrangler init --site` as `usage_model` came after `[site]`.

    [xortive]: https://github.com/xortive
    [issues/1850]: https://github.com/cloudflare/wrangler/pull/1850

## 1.15.1

- ### Features

  - **Add config option to switch usage model to unbound - [ObsidianMinor], [pull/1837]**

    [obsidianminor]: https://github.com/ObsidianMinor
    [pull/1837]: https://github.com/cloudflare/wrangler/pull/1837

- ### Fixes

  - **fix: remove unused import of WasmMainTemplatePlugin - [jasikpark], [pull/1802]**

    This should improve #1721. https://github.com/cloudflare/wrangler/issues/1721#issuecomment-791974664

    [jasikpark]: https://github.com/jasikpark
    [pull/1802]: https://github.com/cloudflare/wrangler/pull/1802

  - **Hot fix for error message helper not working - [Electroid], [pull/1847]**

    The JSON is pretty printed, which means it contains a space.

    [electroid]: https://github.com/Electroid
    [pull/1847]: https://github.com/cloudflare/wrangler/pull/1847

  - **Revert "Print line and column numbers for exception thrown (#1645)" - [Electroid], [pull/1835]**

    This reverts commit 74a89f7c383bc22758cbe55096ce3016c5c319d7.

    Closes #1826

    This commit is causing `wrangler dev` to not show uncaught exceptions. Reverting `chrome-devtools-rs` was also necessary.

    [electroid]: https://github.com/Electroid
    [pull/1835]: https://github.com/cloudflare/wrangler/pull/1835

## 1.15.0

- ### Fixes

  - **fix: remove unused import of WasmMainTemplatePlugin - [jasikpark], [pull/1802]**

    This should improve #1721. https://github.com/cloudflare/wrangler/issues/1721#issuecomment-791974664

    [jasikpark]: https://github.com/jasikpark
    [pull/1802]: https://github.com/cloudflare/wrangler/pull/1802

  - **Revert [pull/1748] - [xortive], [pull/1804]**

    [pull/1748] turned out to interact poorly with workers-sites projects, causing `npm install` to not be run, breaking the build.

    We're going to revert this change. Folks already depending on it should use custom builds ([pull/1677]) once it makes it into a release candidate.

    We incremented the minor version number instead of making a patch release, as this change is more significant than a simple bugfix.

    [xortive]: https://github.com/xortive
    [pull/1748]: https://github.com/cloudflare/wrangler/pull/1748
    [pull/1804]: https://github.com/cloudflare/wrangler/pull/1804
    [pull/1677]: https://github.com/cloudflare/wrangler/pull/1677

## 1.14.1

- ### Fixes

  - **revert default install location change - [xortive], [pull/1798]**

    In 1.14.0, we changed the default install location from `~/.wrangler` to `node_modules`,
    to allow `npx wrangler` to use the pinned version in a project's dependencies. It seems that
    this is causing issues, so we're rolling it back in favor of having folks who want this behavior,
    to specify a the install location in the `config` section of package.json. We'll document this soon.

    [xortive]: https://github.com/xortive
    [pull/1798]: https://github.com/cloudflare/wrangler/pull/1798

## 1.14.0

- ### Features

  - **Display account ID's when possible - [caass], [pull/1786]**

    Previously, users had to log in to their dash to view their account IDs. This PR extracts some of the code used in `whoami` to log their account IDs to the terminal, instead. If for some reason that doesn't work, it'll fall back to telling them to look at the dashboard

    [caass]: https://github.com/caass
    [pull/1786]: https://github.com/cloudflare/wrangler/pull/1786

  - **Feature/monorepo support - [thefill], [pull/1748]**

    Improvement of the detection of node_modules dir - adds bottom-up traversing to find it in parent dirs.

    This should resolve some issues experienced with monorepos (e.g. with nx).

    [thefill]: https://github.com/thefill
    [pull/1748]: https://github.com/cloudflare/wrangler/pull/1748

  - **fix installer - [xortive], [pull/1780]**

    binary-install@0.1.1 broke semver, this PR changes our installer to use a cloudflare-owned version of binary-install with updated dependencies to resolve the previous vulnerability warnings from the old version of axios that was being used.

    This is also a feature, since we now install the `wrangler` binary in `node_modules` instead of `~/.wrangler`.
    This means installing wrangler as a dev dependency works as expected -- `npx wrangler` will run the version
    in your dev dependencies, not the globally installed wrangler.

    Wrangler will now also support setting the `WRANGLER_INSTALL_PATH` environment variable to choose where you install the wrangler binary.
    This environment variable must be set when running wrangler, as well as when installing it.

    [xortive]: https://github.com/xortive
    [pull/1780]: https://github.com/cloudflare/wrangler/pull/1780

  - **kv put with metadata - [ags799], [pull/1740]**

    Closes #1734

    [ags799]: https://github.com/ags799
    [pull/1740]: https://github.com/cloudflare/wrangler/pull/1740
- ### Fixes

  - **don't panic on Client build - [ags799], [pull/1750]**

    This won't fix the issue #1743 but it should give us some more context.

    [ags799]: https://github.com/ags799
    [pull/1750]: https://github.com/cloudflare/wrangler/pull/1750

  - **endlessly retry connection to devtools websocket - [ags799], [pull/1752]**

    Endlessly retry connection to preview's devtools websocket on `wrangler dev`. With exponential backoff.

    Keeps us from panicking in [issue/1510](https://github.com/cloudflare/wrangler/issues/1510).

    [ags799]: https://github.com/ags799
    [pull/1752]: https://github.com/cloudflare/wrangler/pull/1752

  - **fix console formatting - [mdycz], [pull/1749]**

    Fixes #1707

    [mdycz]: https://github.com/mdycz
    [pull/1749]: https://github.com/cloudflare/wrangler/pull/1749

- ### Maintenance

  - **Bump futures-util from 0.3.11 to 0.3.13 - [dependabot], [pull/1778]**

    [dependabot]: https://dependabot.com/
    [pull/1778]: https://github.com/cloudflare/wrangler/pull/1778

  - **Remove extra required_override call from sites ignore logic - [xortive], [pull/1754]**

    [xortive]: https://github.com/xortive
    [pull/1754]: https://github.com/cloudflare/wrangler/pull/1754

  - **remove webpack specific change - [xtuc], [pull/1730]**

    We used to hook directly into webpack internals to rewrite the runtime
    that loads Wasm to make it point to the Worker binding instead of a
    network fetch.

    This change removes the webpack specific change and injects a generic
    runtime to
    ... truncated

    [xtuc]: https://github.com/xtuc
    [pull/1730]: https://github.com/cloudflare/wrangler/pull/1730

  - **Set panic to abort in release mode - [ObsidianMinor], [pull/1762]**

    This should fix cases where we spawn threads that panic and get us in an invalid state that requires us to get killed anyway.

    [obsidianminor]: https://github.com/ObsidianMinor
    [pull/1762]: https://github.com/cloudflare/wrangler/pull/1762

  - **Tweak issue templates - [Electroid], [pull/1776]**

    Made minor edits to the issue templates

    [electroid]: https://github.com/Electroid
    [pull/1776]: https://github.com/cloudflare/wrangler/pull/1776

  - **update binary-install to avoid vulnerable axios version - [simonhaenisch], [pull/1726]**

    Fixes the security warnings when installing the wrangler npm package!

    [simonhaenisch]: https://github.com/simonhaenisch
    [pull/1726]: https://github.com/cloudflare/wrangler/pull/1726

  - **Update README.md for windows install - [koeninger], [pull/1779]**

    note about https://github.com/cloudflare/wrangler/issues/1765

    [koeninger]: https://github.com/koeninger
    [pull/1779]: https://github.com/cloudflare/wrangler/pull/1779

  - **Update release.yml - [xortive], [pull/1783]**

    thanks @rajatsharma for spotting this :)

    [xortive]: https://github.com/xortive
    [pull/1783]: https://github.com/cloudflare/wrangler/pull/1783
## 1.13.0

- ### Features

  - **Add support for text blob bindings - [xortive], [pull/1543], [issue/483]**

    Wrangler now supports text blobs! Text blobs are values to use in your workers, but are read from a file instead of a string in your TOML.

    Usage:

      `text_blobs = { FOO = "path/to/foo.txt", BAR = "path/to/bar.txt" }`

    [pull/1543]: https://github.com/cloudflare/wrangler/pull/1543
    [issue/483]: https://github.com/cloudflare/wrangler/issue/483

- ### Fixes

  - **Support accounts with more than 100 kv namespaces - [ags799], [pull/1717]**

    [pull/1717]: https://github.com/cloudflare/wrangler/pull/1717
    [ags799]: https://github.com/ags799

- ### Maintenance

  - **Remove references to obsolete kv error codes - [ags799], [pull/1727]**

    [pull/1727]: https://github.com/cloudflare/wrangler/pull/1727

## ❗️ 1.12.3

- ### Fixes

  - **Bump OpenSSL version with vulnerability patch  - [pull/1684]**

    [pull/1684]: https://github.com/cloudflare/wrangler/pull/1684

## ❗️ 1.12.2

- ### Fixes

  - **Fix issue which caused `wrangler publish` to nuke sites - [ObsidianMinor], [issue/1625] [pull/1631] [pull/1635]**

    Y'all, we messed up and applied the wrong fix. The change which caused this problem was [this](https://github.com/cloudflare/wrangler/pull/1566).

    [ObsidianMinor]: https://github.com/ObsidianMinor
    [issue/1625]: https://github.com/cloudflare/wrangler/issues/1625
    [pull/1631]: https://github.com/cloudflare/wrangler/pull/1631
    [pull/1635]: https://github.com/cloudflare/wrangler/pull/1635

## ❗️ 1.12.1

- ### Fixes

  - **Revert "allow site to be configured by environment - [nataliescottdavidson], [issue/1625] [pull/1626]**

    previous pr caused an issue.

    [nataliescottdavidson]: https://github.com/nataliescottdavidson
    [issue/1625]: https://github.com/cloudflare/wrangler/issues/1625
    [pull/1626]: https://github.com/cloudflare/wrangler/pull/1626

## ⏰ 1.12.0

- ### Features

  - **Add support for Cron triggers - [ObsidianMinor], [issue/1574] [pull/1592]**

    Cron triggers are a [new Cloudflare Workers feature](https://developers.cloudflare.com/workers/platform/cron-triggers)
    which allow you to schedule execution times to call your workers.

    [ObsidianMinor]: https://github.com/ObsidianMinor
    [pull/1592]: https://github.com/cloudflare/wrangler/pull/1592
    [issue/1574]: https://github.com/cloudflare/wrangler/issues/1574

  - **Structured output for `wrangler publish` - [nataliescottdavidson], [issue/1460] [pull/1538] [pull/1528] [pull/1522]**

    `wrangler publish --output json` produces structured json output which can be parsed with tools such as jq.

    [nataliescottdavidson]: https://github.com/nataliescottdavidson
    [pull/1538]: https://github.com/cloudflare/wrangler/pull/1538
    [pull/1528]: https://github.com/cloudflare/wrangler/pull/1528
    [pull/1522]: https://github.com/cloudflare/wrangler/pull/1522
    [issue/1460]: https://github.com/cloudflare/wrangler/issues/1460

  - **Upload .well-known dotfiles - [nataliescottdavidson], [issue/980] [pull/1566]**

    Wrangler sites users requested the ability to include the .well-known folder
    without including all hidden files.

    [nataliescottdavidson]: https://github.com/nataliescottdavidson
    [pull/1566]: https://github.com/cloudflare/wrangler/pull/1566
    [issue/980]: https://github.com/cloudflare/wrangler/issues/980

  - **Print url for wrangler login - [encadyma], [issue/1544] [pull/1611]**

    [encadyma]: https://github.com/encadyma
    [pull/1611]: https://github.com/cloudflare/wrangler/pull/1611
    [issue/1544]: https://github.com/cloudflare/wrangler/issues/1544

  - **Allow site key to be configured by environment - [oliverpool], [issue/1567] [pull/1573]**

    [oliverpool]: https://github.com/oliverpool
    [pull/1573]: https://github.com/cloudflare/wrangler/pull/1573
    [issue/1567]: https://github.com/cloudflare/wrangler/issues/1567

- ### Fixes

  - **Handle leading slashes in KV keys - [koeninger], [issue/1560] [pull/1559]**

    [koeninger]: https://github.com/koeninger
    [pull/1559]: https://github.com/cloudflare/wrangler/pull/1559
    [issue/1560]: https://github.com/cloudflare/wrangler/issues/1560

- ### Maintenance

  - **Update stalebot settings - [ispivey], [pull/1561]**

    Stalebot now waits 180 days to mark stale, it marks 'timed out' instead of 'wontfix', and added 'never stale' tag.

    [ispivey]: https://github.com/ispivey
    [pull/1561]: https://github.com/cloudflare/wrangler/pull/1561

  - **Pin Rust to 1.47 and fix clippy lints - [ObsidianMinor], [pull/1609]**

    [ObsidianMinor]: https://github.com/ObsidianMinor
    [pull/1609]: https://github.com/cloudflare/wrangler/pull/1609

  - **Copy edit on --host argument description - [thmsdnnr], [issue/1545] [pull/1564]**

    [thmsdnnr]: https://github.com/thmsdnnr
    [pull/1564]: https://github.com/cloudflare/wrangler/pull/1564
    [issue/1545]: https://github.com/cloudflare/wrangler/issues/1545

  - **Hide stderr from browser process - [jspspike], [pull/1516]**

    Wrangler login had the potential to cause random terminal output.

    [jspspike]: https://github.com/jspspike
    [pull/1516]: https://github.com/cloudflare/wrangler/pull/1516

- ### Docs

  - **Instruct users to install Node with nvm - [JasonCoombs], [issue/1517] [pull/1518]**

    Permissions errors occur when users install Node or npm with a different package
    manager.

    [jasoncoombs]: https://github.com/JasonCoombs
    [pull/1518]: https://github.com/cloudflare/wrangler/pull/1518
    [issue/1517]: https://github.com/cloudflare/wrangler/issues/1517

  - **Update links to new docs- [stof] [tuladhar] [ispivey] [rita3ko], [pull/1552] [pull/1532] [pull/1526] [pull/1519]**

    [ispivey]: https://github.com/ispivey
    [stof]: https://github.com/stof
    [tuladhar]: https://github.com/tuladhar
    [rita3ko]: https://github.com/rita3ko
    [pull/1552]: https://github.com/cloudflare/wrangler/pull/1552
    [pull/1532]: https://github.com/cloudflare/wrangler/pull/1532
    [pull/1526]: https://github.com/cloudflare/wrangler/pull/1526
    [pull/1519]: https://github.com/cloudflare/wrangler/pull/1519

  - **Docs for ARM users - [rathboma], [pull/1499]**

    [rathboma]: https://github.com/rathboma
    [pull/1499]: https://github.com/cloudflare/wrangler/pull/1499

## 🌊 1.11.0

- ### Features

  - **New configuration method with `wrangler login` - [jspspike], [pull/1471]**

    `wrangler login` allows you to authenticate Wrangler to use your Cloudflare user credentials without hunting down an API token in the Cloudflare dashboard. It's straightforward! Just run `wrangler login`, enter your credentials, and you're off to the races.

    [jspspike]: https://github.com/jspspike
    [pull/1471]: https://github.com/cloudflare/wrangler/pull/1471

  - **`wrangler dev` now runs on the same machines as your production code - [EverlastingBugstopper] [avidal] [jwheels], [pull/1085]**

    When running `wrangler dev` as an authenticated user, your requests will now run on the same servers that Cloudflare Workers run on in production. This means that what you see is what you get. `wrangler dev` should behave exactly like production, though we still recommend deploying to a staging website before going to production in order to ensure your changes are safe. This change means you get access to things like `request.cf`, the Cache API, and any Cloudflare settings you've applied in the dashboard while developing your Workers.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [avidal]: https://github.com/avidal
    [jwheels]: https://github.com/jwheels
    [pull/1085]: https://github.com/cloudflare/wrangler/pull/1085

  - **`wrangler dev` now supports redirects` - [jspspike], [issue/1508] [pull/1512]**

    When an HTTP response status code is between 300 and 399, the server lets the client know that the data they're looking for isn't here anymore, and the client should issue another separate request to the endpoint denoted in the `Location` header of the response. Before, if you were developing with `wrangler dev` on `example.com`, and your Worker issued a redirect from `https://example.com/oldurl` to `https://example.com/newurl`, that's what would be in the `Location` header. This meant that whatever client you were using would then issue their second request to the public Internet rather than the `wrangler dev` server running on your local machine. With this release, the `Location` header would now be rewritten to `http://127.0.0.1:8787/newurl`, preventing your client from redirecting you away from the Worker you're trying to develop.

    [jspspike]: https://github.com/jspspike
    [pull/1512]: https://github.com/cloudflare/wrangler/pull/1512
    [issue/1508]: https://github.com/cloudflare/wrangler/issues/1508

  - **Add `--config` flag to all commmands to override the default `wrangler.toml` configuration file path - [luanraithz], [issue/1064] [pull/1350]**

    All commands that read from a configuration file can now use a different configuration file path if the `--config` flag is specified. The commands affected are: `kv:namespace`, `kv:key`, `kv:bulk`, `route`, `secret`, `build`, `preview`, `dev`, `publish`, `subdomain`, and `tail`.

    [luanraithz]: https://github.com/luanraithz
    [pull/1350]: https://github.com/cloudflare/wrangler/pull/1350
    [issue/1064]: https://github.com/cloudflare/wrangler/issues/1064

  - **`wrangler dev` configuration options for HTTP protocol - [jspspike], [issue/1204] [pull/1485]**

    `wrangler dev` now takes two additional configuration flags: `--upstream-protocol` and `--local-protocol`. Both of these take a value of either `http` or `https`. `--upstream-protocol` determines what protocol the request going to your preview worker is (previously this was only controlled with the `--host` flag) - this flag defaults to `https`. `--local-protocol` determines what protocol `wrangler dev` listens for and defaults to `http`. If `https` is chosen, a self-signed certificate will be auto-generated for the dev server.

    [jspspike]: https://github.com/jspspike
    [pull/1485]: https://github.com/cloudflare/wrangler/pull/1485
    [issue/1204]: https://github.com/cloudflare/wrangler/issues/1204

  - **`wrangler dev` can be configured in `wrangler.toml` - [jspspike], [issue/1282] [pull/1477]**

    Any flag taken by `wrangler dev` (except `--host`) can now be configured in the `[dev]` section of your `wrangler.toml`. This allows different developers on the same project to share and persist settings for their local development environment.

    [jspspike]: https://github.com/jspspike
    [pull/1477]: https://github.com/cloudflare/wrangler/pull/1477
    [issue/1282]: https://github.com/cloudflare/wrangler/issues/1282

  - **Check if `rustc` is installed before building a Rust project - [ObsidianMinor], [issue/487] [pull/1461]**

    [ObsidianMinor]: https://github.com/ObsidianMinor
    [pull/1461]: https://github.com/cloudflare/wrangler/pull/1461
    [issue/487]: https://github.com/cloudflare/wrangler/issues/487

  - **Improve `preview_id` error message - [EverlastingBugstopper], [issue/1458] [pull/1465]**

    When a `preview_id` is needed, the error message directs the user to add it to their `wrangler.toml`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1465]: https://github.com/cloudflare/wrangler/pull/1465
    [issue/1458]: https://github.com/cloudflare/wrangler/issues/1458

  - **Prevent `wrangler preview` and `wrangler dev` for a Workers Site if there is no authenticated user - [jspspike], [issue/1138] [pull/1447]**

    [jspspike]: https://github.com/jspspike
    [pull/1447]: https://github.com/cloudflare/wrangler/pull/1447
    [issue/1138]: https://github.com/cloudflare/wrangler/issues/1138

- ### Fixes

  - **Fix `wrangler route` commands that take an `--env` flag - [jspspike], [issue/1216] [pull/1448]**

    Before, if you passed an environment to a `wrangler route` command, it wouldn't work properly due to some mishandling of the arguments in the way we used our command line argument parser. This is now fixed and `wrangler route` commands work as expected.

    [jspspike]: https://github.com/jspspike
    [pull/1448]: https://github.com/cloudflare/wrangler/pull/1448
    [issue/1216]: https://github.com/cloudflare/wrangler/issues/1216

  - **Open browser as child process - [jspspike], [issue/516] [pull/1495]**

    When running `wrangler preview`, the browser is now opened as a child process. This fixes an issue on Linux where Wrangler would start the browser and then hang waiting for the browser to exit before it begins watching for changes.

    [jspspike]: https://github.com/jspspike
    [pull/1495]: https://github.com/cloudflare/wrangler/pull/1495
    [issue/516]: https://github.com/cloudflare/wrangler/issues/516

  - **Direct cloudflared output with `wrangler tail` to `/dev/null` - [jspspike], [issue/1432] [pull/1450]**

    [jspspike]: https://github.com/jspspike
    [pull/1450]: https://github.com/cloudflare/wrangler/pull/1450
    [issue/1432]: https://github.com/cloudflare/wrangler/issues/1432

- ### Maintenance

  - **Workers Unlimited is now Workers Bundled - [EverlastingBugstopper], [issue/1466] [pull/1467]**

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1467]: https://github.com/cloudflare/wrangler/pull/1467
    [issue/1466]: https://github.com/cloudflare/wrangler/issues/1466

  - **Replace `assert` with `==` in tests with `assert_eq` - [sudheesh001], [pull/1455]**

    [sudheesh001]: https://github.com/sudheesh001
    [pull/1455]: https://github.com/cloudflare/wrangler/pull/1455

  - **Various typo fixes - [sudheesh001] [jbampton], [pull/1423] [pull/1427] [pull/1428] [pull/1429] [pull/1431] [pull/1443] [pull/1454]**

    [sudheesh001]: https://github.com/sudheesh001
    [jbampton]: https://github.com/jbampton
    [pull/1423]: https://github.com/cloudflare/wrangler/pull/1423
    [pull/1427]: https://github.com/cloudflare/wrangler/pull/1427
    [pull/1428]: https://github.com/cloudflare/wrangler/pull/1428
    [pull/1429]: https://github.com/cloudflare/wrangler/pull/1429
    [pull/1431]: https://github.com/cloudflare/wrangler/pull/1431
    [pull/1443]: https://github.com/cloudflare/wrangler/pull/1443
    [pull/1454]: https://github.com/cloudflare/wrangler/pull/1454

  - **Removed unreachable code in `main.rs` - [luanraithz], [pull/1444]**

    [luanraithz]: https://github.com/luanraithz
    [pull/1444]: https://github.com/cloudflare/wrangler/pull/1444

## 🐹 1.10.3

- ### Features

  - **`wrangler dev` listens on IPv4 by default - [EverlastingBugstopper], [issue/1198] [pull/1405]**

    Before, `wrangler dev` would listen on `[::1]:8787` by default, and call it `localhost` in the terminal output. This was confusing for developers whose `localhost` resolves to IPv4 and not IPv6. Now, `wrangler dev` will listen on `127.0.0.1:8787` by default. This can be overriden by passing values via the `--ip` and `--port` flags.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1405]: https://github.com/cloudflare/wrangler/pull/1405
    [issue/1198]: https://github.com/cloudflare/wrangler/issues/1198

  - **Clarify where to find your `account_id` in the dashboard - [EverlastingBugstopper], [issue/1364] [pull/1395]**

    When you create a new project with `wrangler generate`, it directs you to the Cloudflare Dashboard to find your `account_id` and `zone_id`. However, this flow only worked if you had your own domain. Developers who only use `workers.dev` for their Workers were directed to a page that does not exist! This message now points everyone to a page where they can find the information that they need.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1395]: https://github.com/cloudflare/wrangler/pull/1395
    [issue/1364]: https://github.com/cloudflare/wrangler/issues/1364

- ### Fixes

  - **Allow creation of preview namespaces when a namespace already exists in `wrangler.toml` - [EverlastingBugstopper], [pull/1414]**

    When we introduced KV preview namespaces, we made sure to add nice messages when creating new namespaces so people could easily add the new namespace id to their wrangler.toml in the correct place.

    However, we missed a very common case where developers already have a production namespace defined in their `wrangler.toml` and they want to add a preview namespace. When this is the case, we returned an error message intended to only be thrown when running either wrangler preview or wrangler dev. This is now fixed!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1414]: https://github.com/cloudflare/wrangler/pull/1414

  - **Allow multiple response header values in `wrangler dev` - [EverlastingBugstopper], [issue/1412] [pull/1413]**

    Before, `wrangler dev` would not properly handle response headers that have multiple values. We would iterate over all response headers coming from the Workers runtime, and "insert" them into the header map instead of appending them. This is no longer the case and response headers should now work as expected. More details on this issue can be found [here](https://github.com/cloudflare/wrangler/issues/1412#issuecomment-649764506).

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1413]: https://github.com/cloudflare/wrangler/pull/1413
    [issue/1412]: https://github.com/cloudflare/wrangler/issues/1412

  - **Fix kv-namespace/kv_namespace behavior in environments - [EverlastingBugstopper], [issue/1408] [pull/1409]**

    When KV namespace support was initially added to Wrangler, we documented using `kv-namespaces` in `wrangler.toml`. Unfortunately, the `-` was not consistent with other fields such as `zone_id` and `account_id`, so the decision was made to allow both `kv-namespaces` and `kv_namespaces`. When this change was introduced, it worked with top level `kv_namespaces` entries, but not in environments. This is now fixed! You can now use `kv_namespaces` everywhere you can use `kv-namespaces`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1409]: https://github.com/cloudflare/wrangler/pull/1409
    [issue/1408]: https://github.com/cloudflare/wrangler/issues/1408

- ### Maintenance

  - **Add link to testing docs in CONTRIBUTING.md - [luanraithz], [pull/1416]**

    [luanraithz]: https://github.com/luanraithz
    [pull/1416]: https://github.com/cloudflare/wrangler/pull/1416

  - **Remove unused `Krate::install` code - [EverlastingBugstopper], [issue/247] [pull/1410]**

    When we introduced our own version checking for Wrangler we stopped using `Krate::install`. This PR just removes that unused code.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1410]: https://github.com/cloudflare/wrangler/pull/1410
    [issue/Issue #]: https://github.com/cloudflare/wrangler/issues/247

## 💪 1.10.2

- ### Fixes

  - **Fixes bug preventing `wrangler publish` from deleting over 10000 keys - [ashleymichal], [issue/1398] [pull/1400]**

    In 1.10.0 we removed the limit on bulk KV PUT and DELETE operations by batching the values but missed batched deletes for Workers Sites. Now with some refactoring all batched operations use the same logic and have the same behavior.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1400]: https://github.com/cloudflare/wrangler/pull/1400
    [issue/1398]: https://github.com/cloudflare/wrangler/issues/1398

## ⏳ 1.10.1

- ### Fixes

  - **reinstate longer timeout on bulk uploads for sites - [ashleymichal], [pull/1391]**

      In 1.10.0 we introduced a bug that reduced the timeout for bulk uploads back to the standard 30 seconds. This fixes that and restores the five minute bulk upload/delete timeout.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1391]: https://github.com/cloudflare/wrangler/pull/1391

  - **Increase default timeout to one minute - [EverlastingBugstopper], [pull/1392]**

      For folks with slower connections we're increasing the timeout for API requests.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1392]: https://github.com/cloudflare/wrangler/pull/1392

## ♻️ 1.10.0

- ### Features

  - **`wrangler dev` now requires that you specify "preview" versions of KV namespaces - [EverlastingBugstopper], [ashleymichal], [issue/1032] [pull/1357] [pull/1359] [pull/1360]**

    In order to prevent you from accidentally stomping on production data in Workers KV, we're introducing the concept of explicit *preview namespaces*. When running `wrangler dev`, if you're using Workers KV, you'll need to specify a specific KV Namespace to use when previewing the Worker.

    Specifically, this change:
    * Adds a `preview_id` field to items in `kv_namespaces` in `wrangler.toml` that _must_ be provided in order to preview a worker that has kv namespaces.
    * also adds `--preview` to kv commands in order to interact with them instead of production namespaces.

    If you define a KV Namespace in your `wrangler.toml` but don't specify a `preview_id`, and then try to run `wrangler dev`, you'll see the following:

    ```console
    $ wrangler preview
    💁  JavaScript project found. Skipping unnecessary build!
    Error: In order to preview a worker with KV namespaces, you must designate a preview_id for each KV namespace you'd like to preview.
    ```

    More details can be found in the [documentation](http://localhost:1313/workers/tooling/wrangler/configuration/#kv_namespaces).

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [ashleymichal]: https://github.com/ashleymichal
    [pull/1357]: https://github.com/cloudflare/wrangler/pull/1357
    [pull/1359]: https://github.com/cloudflare/wrangler/pull/1359
    [pull/1360]: https://github.com/cloudflare/wrangler/pull/1360
    [issue/1032]: https://github.com/cloudflare/wrangler/issues/1032

  - **`wrangler subdomain` allows you to rename your existing workers.dev subdomain - [nprogers], [issue/1279] [pull/1353]**

    Workers recently introduced the ability to rename your workers.dev subdomain in the Workers dashboard. This change allows you to do the same thing from `wrangler`.

    [nprogers]: https://github.com/nprogers
    [pull/1353]: https://github.com/cloudflare/wrangler/pull/1353
    [issue/1279]: https://github.com/cloudflare/wrangler/issues/1279

  - **Return more detailed error when preview uploads fail - [jahands], [issue/1330] [pull/1356]**

    When running `wrangler dev` or `wrangler preview`, if the upload failed for validation or other reasons, we displayed pretty obtuse errors. Now we pass along more informative error details.

    [jahands]: https://github.com/jahands
    [pull/1356]: https://github.com/cloudflare/wrangler/pull/1356
    [issue/1330]: https://github.com/cloudflare/wrangler/issues/1330

  - **Check for wrangler updates every 24 hours and message if you should update - [EverlastingBugstopper], [jspspike], [issue/397] [pull/1190] [pull/1331]**

    wrangler will now let you know if there's an update available, and will only bug you once every 24 hours.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [jspspike]: https://github.com/jspspike
    [pull/1190]: https://github.com/cloudflare/wrangler/pull/1190
    [pull/1331]: https://github.com/cloudflare/wrangler/pull/1331
    [issue/397]: https://github.com/cloudflare/wrangler/issues/397

  - **Improve some error messages with human-readable suggestions - [jspspike], [issue/1157] [pull/1329]**

    Previously, you would see not-super-helpful error messages if your API Token was expired or missing some permissions, didn't have Workers Unlimited enabled and tried to upload to KV, or tried to create a namespace that already existed. But we strive for helpful, informative error messages!

    Now, you'll see the following error messages as appropriate:
    ```
    10026 => "You will need to enable Workers Unlimited for your account before you can use this feature.",
    10014 => "Namespace already exists, try using a different namespace.",
    10037 => "Edit your API Token to have correct permissions, or use the 'Edit Cloudflare Workers' API Token template.",
    ```

    [jspspike]: https://github.com/jspspike
    [pull/1329]: https://github.com/cloudflare/wrangler/pull/1329
    [issue/1157]: https://github.com/cloudflare/wrangler/issues/1157

- ### Fixes

  - **wrangler dev pretty-prints JSON messages from console.log() - [jspspike], [issue/1249] [pull/1371]**

    Previously, when running `wrangler dev`, you could `console.log(JSON.stringify(some_object))`, but the output was hard to read: newlines weren't appropriately parsed and whitespace was a bit of a mess.

    This change pretty-prints JSON received from the Inspector, so JSON output is much easier to read in `wrangler dev`.

    [jspspike]: https://github.com/jspspike
    [pull/1371]: https://github.com/cloudflare/wrangler/pull/1371
    [issue/1249]: https://github.com/cloudflare/wrangler/issues/1249

  - **Don't install wasm-pack when publishing a type:webpack project - [EverlastingBugstopper], [issue/745] [pull/1344]**

    We shouldn't install wasm-pack if your project doesn't need it. We thought we fixed this in 1.7.0, but we didn't. This time it's fixed for real, we swear.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1344]: https://github.com/cloudflare/wrangler/pull/1344
    [issue/745]: https://github.com/cloudflare/wrangler/issues/745

  - **Pin wasm-pack and cargo-generate to specific versions instead of always downloading the latest version - [jspspike], [issue/1240] [pull/1358]**

    Previously, every time `wasm-pack` or `cargo-generate` was needed, Wrangler would download and install the latest version. Now, we pin those dependencies to a specific version in every release of Wrangler. If you have either of these tools installed locally, Wrangler will use them if you have a more recent version.

    [jspspike]: https://github.com/jspspike
    [pull/1358]: https://github.com/cloudflare/wrangler/pull/1358
    [issue/1240]: https://github.com/cloudflare/wrangler/issues/1240

  - **Support batched KV PUT and DELETE operations to improve Workers Site publish performance - [jspspike], [issue/1191] [pull/1339]**

    Previously, you couldn't use Workers Sites if you wanted to upload more than 10,000 static files, and it took a lot of time to upload close to that many files. This change instead batches upload and delete calls, allowing us to increase the limit and improve performance for everyone.

    [jspspike]: https://github.com/jspspike
    [pull/1339]: https://github.com/cloudflare/wrangler/pull/1339
    [issue/1191]: https://github.com/cloudflare/wrangler/issues/1191

- ### Maintenance

  - **Add Code of Conduct - [EverlastingBugstopper], [pull/1346]**

    We actually didn't have one before, so now we do!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1346]: https://github.com/cloudflare/wrangler/pull/1346

  - **Deprecate undocumented KV `bucket` attribute - [ashleymichal], [issue/1136] [pull/1355]**

    The `bucket` attribute was not originally intended to be released into the wild. We announced deprecation in March, and are now removing it as an available configuration property.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1355]: https://github.com/cloudflare/wrangler/pull/1355
    [issue/1136]: https://github.com/cloudflare/wrangler/issues/1136

  - **Refactor to separate KV commands, implementation of said commands, and site-specific logic - [ashleymichal], [pull/1332]**

    This refactor makes the logic for interacting with KV more re-usable and maintainable.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1332]: https://github.com/cloudflare/wrangler/pull/1332

  - **Add SECURITY.md with responsible reporting guidelines - [EverlastingBugstopper], [pull/1345]**

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1345]: https://github.com/cloudflare/wrangler/pull/1345

## 🐼 1.9.2

- ### Fixes

  - **Fix piping secret values to `wrangler secret put <VAR_NAME>` - [dmcgowan], [issue/1322] [pull/1316]**

    In 1.9.1, we introduced a bug where piping values to `wrangler secret put` no longer worked. In 1.9.2, that bug is squashed, and the command works as expected.

    [dmcgowan]: https://github.com/dmcgowan
    [pull/1316]: https://github.com/cloudflare/wrangler/pull/1316
    [issue/1322]: https://github.com/cloudflare/wrangler/issues/1322

## 🐎 1.9.1

- ### Features

  - **Accept --verbose for every command - [bradyjoslin], [issue/975] [pull/1110]**

    Not every command outputs additional information when you pass `--verbose`, but none of them will fail to run if you pass `--verbose` after this change.

    [bradyjoslin]: https://github.com/bradyjoslin
    [pull/1110]: https://github.com/cloudflare/wrangler/pull/1110
    [issue/975]: https://github.com/cloudflare/wrangler/issues/975

  - **`wrangler dev` checks if the specified port is available - [EverlastingBugstopper], [issue/1122] [pull/1272]**

    When starting up `wrangler dev`, it now checks to see if the requested port is already in use and returns a helpful error message if that's the case.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1272]: https://github.com/cloudflare/wrangler/pull/1272
    [issue/1122]: https://github.com/cloudflare/wrangler/issues/1122

- ### Fixes

  - **Don't reinstall vendored binaries on every build - [EverlastingBugstopper], [issue/768] [pull/1003]**

    You may have noticed some very verbose and over-eager installation output when running Wrangler. Every `webpack` type build would install `wranglerjs` and `wasm-pack`. This was... super annoying and not a great experience, especially when running `wrangler preview --watch` or `wrangler dev`. Each time you'd change a file, Wrangler would reinstall those external dependencies. This doesn't happen anymore! Wrangler will still download and install these external dependencies, but only if you have an outdated version.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1003]: https://github.com/cloudflare/wrangler/pull/1003
    [issue/768]: https://github.com/cloudflare/wrangler/issues/768

  - **Remove redundant builds - [EverlastingBugstopper], [issue/1219] [pull/1269]**

    When running `wrangler preview --watch` or `wrangler dev` on a `webpack` type project, Wrangler will provide a new build artifact and upload it via the Cloudflare API. Before, we'd start a long-running `webpack --watch` command, _in addition to_ running `webpack` on every change. We were running two builds on every change! This was not great and has been removed. This, combined with the above fix removing redundant installations, should greatly improve your dev iteration cycles.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1269]: https://github.com/cloudflare/wrangler/pull/1269
    [issue/1219]: https://github.com/cloudflare/wrangler/issues/1219

  - **`wrangler dev` will reconnect to the devtools WebSocket after being disconnected - [EverlastingBugstopper], [issue/1241] [pull/1276]**

    `wrangler dev` initiates a WebSocket connection via the Cloudflare API in order to stream `console.log` messages to your terminal. Over time, it's very likely that the WebSocket would be disconnected. When this happened, Wrangler would panic, requiring developers to restart the process. Now, if `wrangler dev` gets disconnected, it will issue a reconnect request, allowing developers to run `wrangler dev` as long as they are connected to the Internet.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1276]: https://github.com/cloudflare/wrangler/pull/1276
    [issue/1241]: https://github.com/cloudflare/wrangler/issues/1241

- ### Maintenance

  - **Adds `Developing Wrangler` section to `CONTRIBUTING.md` - [EverlastingBugstopper], [issue/270] [pull/1288]**

    We love external contributors, and what better way to help get folks kickstarted than to add some documentation on developing Wrangler? Check out [CONTRIBUTING.md](./CONTRIBUTING.md) if you're interested in helping out.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1288]: https://github.com/cloudflare/wrangler/pull/1288
    [issue/270]: https://github.com/cloudflare/wrangler/issues/270

  - **Remove `--release` from `wrangler publish --help` - [EverlastingBugstopper], [pull/1289]**

    We deprecated `wrangler publish --release` a long time ago in favor of environments, but it's still an accepted argument to preserve backwards compatibility. Now, it no longer shows up in `wrangler publish --help` as an accepted argument, even though it's still an alias of `wrangler publish`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1289]: https://github.com/cloudflare/wrangler/pull/1289

  - **Updates license file to wrangler@cloudflare.com - [EverlastingBugstopper], [pull/1290]**

    The copyright in our MIT license was outdated and pointed to the email address of @ashleygwilliams (who no longer works at Cloudflare 😢). Now it points to [wrangler@cloudflare.com](mailto:wrangler@cloudflare.com) :)

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1290]: https://github.com/cloudflare/wrangler/pull/1290

  - **Add Dependabot to Wrangler - [ispivey], [pull/1294]**

    Wrangler now uses [Dependabot](https://dependabot.com/) to automatically update dependencies. We had already been doing this on a sort of ad-hoc basis, but this should make it much easier to stay on top of updates!

    [ispivey]: https://github.com/ispivey
    [pull/1294]: https://github.com/cloudflare/wrangler/pull/1294

  - **Remove unused code warnings - [ashleymichal], [pull/1304]**

    For our integration tests we create fixtures containing sample Workers projects. When we ran `cargo test`, `cargo` would say that the code used to create said fixtures were unused (which was not true). This PR moves the fixture code to a place where `cargo` says "This is a Fine Place for This Code."

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1304]: https://github.com/cloudflare/wrangler/pull/1304

  - **Handle clippy warnings - [ashleymichal] [EverlastingBugstopper], [pull/1305] [pull/1306]**

    `cargo clippy` is a helpful little tool that helps you write more idiomatic Rust. Over time, we've developed an immunity to the warnings produced by this tool, and we took a stab at cleaning some of them up.

    [ashleymichal]: https://github.com/ashleymichal
    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1305]: https://github.com/cloudflare/wrangler/pull/1305
    [pull/1306]: https://github.com/cloudflare/wrangler/pull/1306

## 🦚 1.9.0

- ### Features

  - **Wrangler Tail - [ashleymichal], [gabbifish], [EverlastingBugstopper], [pull/1182]**

    Wrangler Tail introduces a way to view console statements and exceptions live as they occur in your Worker. Simply run `wrangler tail` against any deployed Worker and pipe the output through `jq` or to a file to stream trace events per request.

    [ashleymichal]: https://github.com/ashleymichal
    [gabbifish]: https://github.com/gabbifish
    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1182]: https://github.com/cloudflare/wrangler/pull/1182

  - **Much faster build times for Workers Sites projects - [EverlastingBugstopper], [pull/1221]**

    When you deploy a Workers Site, Wrangler generates a unique hash for each file. It does this so that your Worker does not serve stale files from Cloudflare's edge cache to end users. Unfortunately, generating these hashes took a really really long time since we were using a cryptographically strong hash. Since we're just using this hash for cache invalidation, we decided it's not necessary to use such a complicated algorithm. We switched to using [xxhash](https://github.com/Cyan4973/xxHash) and have seen noticeable speed improvements.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1221]: https://github.com/cloudflare/wrangler/pull/1221

  - **Add --url to wrangler preview - [larkin-nz], [issue/351] [pull/1001]**

    `wrangler preview` now has the ability to open any URL! Before, `wrangler preview` would always open `example.com`, and you would be able to change the URL in the browser only. Now, you can use `wrangler preview --url https://mysite.com/my-amazing-endpoint` and your preview session will get started off on the right foot.

    [larkin-nz]: https://github.com/larkin-nz
    [pull/1001]: https://github.com/cloudflare/wrangler/pull/1001
    [issue/351]: https://github.com/cloudflare/wrangler/issues/351

  - **Print email addresses for API token users on `wrangler whoami` - [dhaynespls], [issue/863] [pull/1212]**

    Before, if you ran `wrangler whoami` as an API token user, you didn't get much info. Due to some heavy lifting by folks working on the Cloudflare API, API token users with the correct permissions can now see what email address they are authenticated with when they run `wrangler whoami`. Nifty!

    [dhaynespls]: https://github.com/dhaynespls
    [pull/1212]: https://github.com/cloudflare/wrangler/pull/1212
    [issue/863]: https://github.com/cloudflare/wrangler/issues/863

  - **`wrangler generate` auto increments default worker name - [xprazak2], [issue/58] [pull/469]**

    When `wrangler generate` is run without a name for the worker, it will find a name for the worker that does not already exist in that directory.

    [xprazak2]: https://github.com/xprazak2
    [pull/469]: https://github.com/cloudflare/wrangler/pull/469
    [issue/58]: https://github.com/cloudflare/wrangler/issues/58

  - **Standardize colors in `stdout` - [EverlastingBugstopper], [pull/1248]**

    Wrangler likes to print colors where appropriate, and now there is a standard module for printing different colors that is used across the codebase.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1248]: https://github.com/cloudflare/wrangler/pull/1248

  - **Suggests `wrangler init` if `wrangler.toml` does not exist - [ashleymichal], [issue/827] [pull/1239]**

    When there is no `wrangler.toml` in a directory you're trying to run Wrangler in, it doesn't know what to do. The way to fix this is to make a `wrangler.toml`, and the way to do that is to run `wrangler init`.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1239]: https://github.com/cloudflare/wrangler/pull/1239
    [issue/827]: https://github.com/cloudflare/wrangler/issues/827

- ### Fixes

  - **Allow kv-namespaces and kv_namespaces - [EverlastingBugstopper], [issue/1158] [pull/1169]**

    Most fields defined in `wrangler.toml` are one word, but some of them are two! In the past, we usually use `_` to separate words, but somehow we used a `-` for `kv-namespaces`. This was inconsistent and a bit confusing. Now we allow both for the sake of backwards compatibility, but in the future we'll try to stick to `snake_case`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1169]: https://github.com/cloudflare/wrangler/pull/1169
    [issue/1158]: https://github.com/cloudflare/wrangler/issues/1158

  - **Typo fix in `wrangler init` - [jplhomer], [pull/1210]**

    A successful `wrangler init` execution used to output "Succesfully" instead of "Successfully", but not anymore!

    [jplhomer]: https://github.com/jplhomer
    [pull/1210]: https://github.com/cloudflare/wrangler/pull/1210

  - **More granular errors in `wrangler dev` - [EverlastingBugstopper], [pull/1251]**

    In the last release we added an error message in `wrangler dev` for failed uploads. Unfortunately it was a bit overeager and some information about different types of errors were lost. This behavior has been fixed!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1251]: https://github.com/cloudflare/wrangler/pull/1251

- ### Maintenance

  - **Unify wrangler's user agent - [EverlastingBugstopper], [issue/731] [pull/1070]**

    Wrangler sure does send a lot of API requests! Before, about half of the API requests Wrangler sent would send them with the HTTP header `User-Agent: wrangler`. Now, all requests sent by Wrangler include that User Agent. This lets the APIs we use know that the request is coming from this tool. Yay for being good [netizens](https://www.merriam-webster.com/dictionary/netizen)!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1070]: https://github.com/cloudflare/wrangler/pull/1070
    [issue/731]: https://github.com/cloudflare/wrangler/issues/731

  - **Refactors and documentation of `wrangler dev` - [EverlastingBugstopper], [pull/1220]**

    No behavior changes with this one, just some improvements to code layout and some extra documentation comments. Check it out if you're interested!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1220]: https://github.com/cloudflare/wrangler/pull/1220

## 🎭 1.8.4

- ### Fixes

  - **Don't remove user configuration on npm installs - [EverlastingBugstopper], [issue/1180] [pull/1181]**

    Wrangler started removing user's authentication configuration files on reinstallation from npm - this is no good and is fixed in this release.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1181]: https://github.com/cloudflare/wrangler/pull/1181
    [issue/1180]: https://github.com/cloudflare/wrangler/issues/1180

  - **Allow multiline files to be piped to `wrangler secret put` - [EverlastingBugstopper], [issue/1132] [pull/1171]**

    Previously, if you tried to pipe a multiline file to `wrangler secret put`, the secret would only upload the first line of the file. This... was not helpful - `cat hello_world.txt | wrangler secret put` should behave as expected with this release.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1171]: https://github.com/cloudflare/wrangler/pull/1171
    [issue/1132]: https://github.com/cloudflare/wrangler/issues/1132

- ### Maintenance

  - **Bump GitHub Actions checkout version - [imbsky], [pull/1170]**

    GitHub Actions are pretty nifty, and we've started using them as our CI provider in Wrangler. Actions allow you to specify a step that "uses" a template, and one of the most used templates is the template that checks out relevant code. GitHub just released v2 of that template, and our CI now uses it!

    [imbsky]: https://github.com/imbsky
    [pull/1170]: https://github.com/cloudflare/wrangler/pull/1170

## 🍟 1.8.3

- ### Features

  - **Improvements to the Workers Sites asset manifest - [EverlastingBugstopper], [issue/897] [pull/1145]**

    Workers Sites uses the concept of an asset manifest to invalidate Cloudflare's cache when new files are published. Every time you publish your Workers Site, Wrangler will re-create a mapping of file names with a hash to the contents of the file. This release includes a few steps that improve this experience:

    - Manifest sizes are smaller by a magnitude of ~6.4. This should help some folks who were previously running into size issues when uploading a Workers Site.

    - Any time an asset manifest is created, you will see the files that are being hashed in real time with a fancy loading spinner - no more waiting without any information!

    - Asset manifest creation is now faster due to a  refactor.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1145]: https://github.com/cloudflare/wrangler/pull/1145
    [issue/897]: https://github.com/cloudflare/wrangler/issues/897

  - **Clarify mutual exclusivity of zoneless v. zoned deploys - [EverlastingBugstopper], [issue/1152] [pull/1154]**

    When publishing a Worker, you must specify either `workers_dev = true` or both a `zone_id` and `route/routes`. Previously, if your `wrangler.toml` violated this requirement, it would error with the following message:

    ```console
    $ wrangler publish
    Error: you must set workers_dev = true OR provide a zone_id and route/routes.
    ```

    It's technically correct, but we can make it even more clear what the issue is. The new error message looks like:

    ```console
    $ wrangler publish
    Error: you must set EITHER workers_dev = true OR provide a zone_id and route/routes.
    ```

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1154]: https://github.com/cloudflare/wrangler/pull/1154
    [issue/1152]: https://github.com/cloudflare/wrangler/issues/1152

- ### Fixes

  - **Fixes `wrangler config` information message - [EverlastingBugstopper], [pull/1164]**

    In Wrangler 1.8.2, we updated the formatting of some of Wrangler's informational messages. Unfortunately when this was introduced, it came with a bug in `wrangler config` that made the message read out in the wrong order. This is fixed in this release!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1164]: https://github.com/cloudflare/wrangler/pull/1164

- ### Maintenance

  - **Remove unused badges from README - [EverlastingBugstopper], [pull/1166]**

    We no longer use Azure Pipelines as our CI provider, nor do we run non-test builds in CI so we removed those badges from the README.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/1166]: https://github.com/cloudflare/wrangler/pull/1166

## 🐈 1.8.2

- ### Features

  - **Configurable binary host URL - [noherczeg], [pull/1018]**

    Previously, binaries installed by Wrangler were all assumed to come from npm. If you work in a controlled environment and can only install binaries from a specific endpoint (instead of npm), you can now specify that endpoint using the WRANGLER_BINARY_HOST environment variable.

    [pull/1018]: https://github.com/cloudflare/wrangler/pull/1018
    [noherczeg]: https://github.com/noherczeg

- ### Fixes

  - **Eliminate downtime when redeploying Workers Sites - [ashleymichal], [issue/783], [pull/1115]**

    When Workers Sites were first introduced, redeploying a site could lead to a few seconds of downtime if the Worker upload fails. Specifically, if a new Workers Sites upload failed, it was possible that the old, now-unused files in Workers KV would be deleted anyways, meaning that the preexisting Workers Site would suddenly have missing resources. This fix waits to delete now-unused files until after a new Workers Sites script is published.

    [issue/783]: https://github.com/cloudflare/wrangler/issues/783
    [pull/1115]: https://github.com/cloudflare/wrangler/pull/1115
    [ashleymichal]: https://github.com/ashleymichal

- ### Maintenance

  - **Add npm badge to README - [tomByrer], [pull/1121]**

    Add badge to README that points to npm page for Wrangler.

    [pull/1115]: https://github.com/cloudflare/wrangler/pull/1121
    [tomByrer]: https://github.com/tomByrer

  - **Unify attention-grabbing messages - [EverlastingBugstopper], [pull/1128]**

    Use more actionable, easy-to-read information printouts throughout Wrangler.

    [pull/1115]: https://github.com/cloudflare/wrangler/pull/1128
    [tomByrer]: https://github.com/EverlastingBugstopper

## 😈 1.8.1

- ### Features

  - **Error messaging for internet required to talk to Cloudflare API - [EverlastingBugstopper], [issue/1093] [pull/1114]**

    With the release of `wrangler dev` in 1.8.0, it was not clear to users that internet is required since the feature communicates with Cloudflare's API. With this error message, users without internet connection are shown actionable next steps - check internet connection and lastly check if Cloudflare's API is down.

    [issue/1093]: https://github.com/cloudflare/wrangler/issues/1093
    [pull/1114]: https://github.com/cloudflare/wrangler/pull/1114
    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper

- ### Fixes

  - **Fix live reload for `wrangler dev` - [EverlastingBugstopper], [issue/1082] [pull/1117]**

    `wrangler dev` re-builds and re-uploads your script to the Cloudflare API when it detects a file change. The Cloudflare API returns a new token which allows `wrangler dev` to route subsequent requests to the new script. Previously, `wrangler dev` would re-build, re-upload, and receive the new token, but it wouldn't use it for a couple of minutes due to some faulty threading logic. (darn mutexes!) After this change, `wrangler dev` will block incoming requests when it is switching the token, thus fixing the issue.

    [issue/1082]: https://github.com/cloudflare/wrangler/issues/1082
    [pull/1117]: https://github.com/cloudflare/wrangler/pull/1117
    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper

  - **Remove unneeded carriage return in `wrangler secret put` - [gabbifish], [issue/1109] [pull/1112]**

    Previously, interactive input from `wrangler secret put` added a carriage return to the secret key/value pairs on Windows. This no longer happens and input is parsed properly before uploading.

    [issue/1109]: https://github.com/cloudflare/wrangler/issues/1109
    [pull/1112]: https://github.com/cloudflare/wrangler/pull/1112
    [gabbifish]: https://github.com/gabbifish

## 🙊 1.8.0

- ### Features

  - **`wrangler dev` - [EverlastingBugstopper], [issue/845] [pull/883]**

    `wrangler dev` is a local proxy server to Cloudflare's preview service, allowing you to automatically re-build and preview your application on `localhost`. This feature is in alpha and we're looking for feedback and bug reports: check out [this issue](https://github.com/cloudflare/wrangler/issues/1047)!

    `wrangler dev` works very similarly to `wrangler preview`, but instead of opening your browser to preview your Worker, it will start a server on `localhost` that will execute your Worker on incoming HTTP requests:

    ```
    $ wrangler dev
    ```

    You should be able to send HTTP requests to `localhost:8787`, along with any headers or other request data, and your Worker should execute as expected. Additionally, you'll see `console.log` messages and exceptions appearing in your terminal (!!!).

    For more information on `wrangler dev`'s options, such as passing a custom `host`, `ip`, or `port`, run `wrangler dev` in your terminal for the available flags and options.

    [issue/845]: https://github.com/cloudflare/wrangler/issues/845
    [pull/883]: https://github.com/cloudflare/wrangler/pull/883

  - **Multi-route support - [ashleymichal], [issue/866] [pull/916]**

    Wrangler now allows developers to publish their Workers projects to multiple routes on a Cloudflare zone.

    To deploy your Workers project to multiple routes, you can migrate from the `route` key to `routes`:

    ```toml
    name = "worker"
    type = "javascript"
    account_id = "youraccountid"
    # change this line
    # route = "example.com/foo/*"
    # to this line
    routes = ["example.com/foo/*", "example.com/bar/*"]
    zone_id = "yourzoneid"
    ```

    [issue/866]: https://github.com/cloudflare/wrangler/issues/866
    [pull/916]: https://github.com/cloudflare/wrangler/pull/916

  - **`wrangler secret` commands - [ashleymichal], [bradyjoslin], [issue/907] [issue/909] [issue/912] [pull/1045]**

    Wrangler now allows developers to use **secrets** in their Workers codebase. Secrets are secure values that can be accessed as constants, similar to text variables, inside of your Workers code.

    To set a secret, you can use `wrangler secret put MY_SECRET_NAME`. The interactive prompt will ask for the secret text you'd like to add to your project:

    ```bash
    $ wrangler secret put MY_SECRET_NAME
    Enter the secret text you'd like assigned to the variable MY_SECRET_NAME on the script named my-project
    ```

    Importantly, secrets are constrained to an environment, and do not carry over between different deployed Workers (e.g. `my-worker` and `my-worker-production`). This allows you to use different API keys, URLs, and other common "environment variable"-style values in your different environments. Specifying an environment can be done using the `--env` (or `-e`, for short):

    ```bash
    $ wrangler secret put MY_SECRET_NAME --env production
    Enter the secret text you'd like assigned to the variable MY_SECRET_NAME on the script named my-project-production
    ```

    The `wrangler secret` subcommand also allows developers to `list` and `delete` secrets for your Workers projects:

    ```bash
    $ wrangler secret delete MY_SECRET_NAME
    Are you sure you want to permanently delete the variable MY_SECRET_NAME on the script named my-project [y/n] y
    🌀  Deleting the secret MY_SECRET_NAME on script my-project.
    ✨  You've deleted the secret MY_SECRET_NAME.

    $ wrangler secret list
    [{"name":"API_KEY","type":"secret_text"},{"name":"MY_OTHER_SECRET","type":"secret_text"}]
    ```

    [issue/907]: https://github.com/cloudflare/wrangler/issues/907
    [issue/909]: https://github.com/cloudflare/wrangler/issues/909
    [issue/912]: https://github.com/cloudflare/wrangler/issues/912
    [pull/1045]: https://github.com/cloudflare/wrangler/pull/1045
    [issue/1100]: https://github.com/cloudflare/wrangler/issues/1100
    [pull/1101]: https://github.com/cloudflare/wrangler/pull/1101

  - **Plain text binding support - [EverlastingBugstopper] - [issue/993] [pull/1014]**

    In addition to secrets, Wrangler now also supports setting "plain text" bindings – values that will be available as constants in your Workers code, but aren't encrypted. This can be done by passing values in `wrangler.toml` under the `vars` key:

    ```toml
    name = "worker"
    type = "javascript"
    account_id = "your-account-id"
    workers_dev = true
    vars = { ENV = "staging" }
    [env.prod]
    vars = { ENV = "production" }
    ```

    [issue/993]: https://github.com/cloudflare/wrangler/issues/993
    [pull/1014]: https://github.com/cloudflare/wrangler/pull/1014

  - **Return accounts and account IDs when running `wrangler whoami` - [ashleygwilliams], [issue/630] [pull/983]**

    We've made big improvements to `wrangler whoami`, and now return a list of Cloudflare accounts and account IDs for your authenticated user. If you are unauthenticated, or something is wrong with your API key or token, we'll also return an error with this command to help you understand how to fix your authentication issues!

    ![Preview](https://user-images.githubusercontent.com/1163554/71917894-c624e580-3146-11ea-9793-1e8f8a92a4ea.png)

    [issue/630]: https://github.com/cloudflare/wrangler/issues/630
    [pull/983]: https://github.com/cloudflare/wrangler/pull/983

  - **Configure sourcemap file - [xtuc], [issue/681] [pull/1063]**

    `webpack` (by default) emits a sourcemap that maps to a `main.js` file, which doesn't match the Workers runtime's configured filename, `worker.js`. This causes exception reporting tools to be unable to process a Workers sourcemap file – we've updated our Webpack config to output the file `worker.js` and have fixed this issue.

    [issue/681]: https://github.com/cloudflare/wrangler/issues/681
    [pull/1063]: https://github.com/cloudflare/wrangler/pull/1063

  - **Upload "draft" worker if secret is created before initial worker script has been uploaded - [gabbifish], [issue/913] [pull/1087]**

    If your script hasn't yet been deployed to the Workers platform, creating and deleting secrets will also create a "draft" Worker – allowing you to still manage secret bindings before you deploy the first version of your script.

    [issue/913]: https://github.com/cloudflare/wrangler/issues/913
    [pull/1087]: https://github.com/cloudflare/wrangler/pull/1087

- ### Maintenance

  - **Correctly tar release binaries - [EverlastingBugstopper], [issue/1055] [pull/1062]**

    This PR updates the way that release binaries are generated during Wrangler's release workflow.

    [issue/1055]: https://github.com/cloudflare/wrangler/issues/1055
    [pull/1062]: https://github.com/cloudflare/wrangler/pull/1062

  - **Change NPM binary permissions - [xtuc], [pull/1058]**

    This PR removes an unnecessary executable permission from `npm/binary.js`.

    [pull/1058]: https://github.com/cloudflare/wrangler/pull/1058

  - **Improvements to GitHub Actions build process - [EverlastingBugstopper], [pull/1037]**

    This PR adds a number of improvements to wrangler's GitHub Actions workflows, including caching, release management, and more granular trigger conditions.

    [pull/1037]: https://github.com/cloudflare/wrangler/pull/1037

  - **Add GitHub Actions badge to README - [EverlastingBugstopper], [pull/1030]**

    This PR adds a GitHub Actions badge to our README, indicating whether the repo's builds are currently passing:

    [![GitHub Actions - Test Status](https://github.com/cloudflare/wrangler/workflows/Rust%20Tests/badge.svg)](https://github.com/cloudflare/wrangler/actions)

    [pull/1030]: https://github.com/cloudflare/wrangler/pull/1030

  - **Test Rust with GitHub Actions - [EverlastingBugstopper], [pull/1028]**

    This PR adds a GitHub Actions workflow for running `wrangler`'s test suite on a number of platforms and Rust versions.

    [pull/1028]: https://github.com/cloudflare/wrangler/pull/1028

  - **Add release checklist - [EverlastingBugstopper], [pull/1021]**

    This PR adds a release checklist, documenting the steps that we use to release new versions of Wrangler. That checklist includes writing this CHANGELOG - very meta!!!

    [pull/1021]: https://github.com/cloudflare/wrangler/pull/1021

  - **Update dependencies - [EverlastingBugstopper], [pull/1000]**

    This PR updates some project dependencies as a result of running `cargo update`.

    [pull/1000]: https://github.com/cloudflare/wrangler/pull/1000

  - **Run CI on pull requests, not pushes - [EverlastingBugstopper], [pull/1090]**

    This PR changes the GitHub Actions workflow "event trigger" to fire on `pull_request`, not `push`. This will allow wrangler's GitHub Actions workflows to run on PRs sent from forks!

    [pull/1090]: https://github.com/cloudflare/wrangler/pull/1090

  - **Zip .tar files in CI - [EverlastingBugstopper], [pull/1069] [pull/1080]**

    These PRs fix some issues in the GitHub Actions release workflow that were causing release artifacts to be incorrectly generated.

    [pull/1080]: https://github.com/cloudflare/wrangler/pull/1080
    [pull/1069]: https://github.com/cloudflare/wrangler/pull/1069

  - **Fixes clippy warnings - [EverlastingBugstopper], [pull/1071]**

    This PR fixes some linting issues surfaced by clippy throughout the project.

    [pull/1071]: https://github.com/cloudflare/wrangler/pull/1071

  - **Extract upload and deploy to lib modules - [ashleymichal], [pull/1075]**

    This PR refactors some of the underlying code used inside of `wrangler publish`, to create two higher-level `upload` and `deploy` modules. This work has already been used to support "draft workers" in #1087, and to reduce duplication of code between `wrangler preview`, `wrangler dev`, and `wrangler publish`.

    [pull/1075]: https://github.com/cloudflare/wrangler/pull/1075

## 💬 1.7.0

- ### Features

  - **Do not factor in .gitignore into workers sites upload directory traversal - [gabbifish], [issue/958] [pull/981]**

    This change ensures that the wrangler include/exclude logic for Workers Sites bucket directory traversal does NOT take into account .gitignore, since best practice for static site generators is to put your build directory into your .gitignore.

    [gabbifish]: https://github.com/gabbifish
    [pull/981]: https://github.com/cloudflare/wrangler/pull/981
    [issue/958]: https://github.com/cloudflare/wrangler/issues/958

  - **Update cloudflare-rs, reqwest, http, uuid - [ashleymichal], [issue/301] [pull/1009]**

    These dependency updates may look like routine maintenance, but this reqwest version actually makes support for corporate proxies possible!

    [ashleymichal]: https://github.com/ashleymichal
    [pull/1009]: https://github.com/cloudflare/wrangler/pull/1009
    [issue/301]: https://github.com/cloudflare/wrangler/issues/301

  - **Add progress bar during Site upload - [gabbifish], [issue/906] [pull/956]**

    Larger static asset uploads in Wrangler now show a progress bar based on the bulk uploads being made.

    [gabbifish]: https://github.com/gabbifish
    [pull/956]: https://github.com/cloudflare/wrangler/pull/956
    [issue/906]: https://github.com/cloudflare/wrangler/issues/906

  - **Allow custom webpack config for Workers Sites projects - [ashleymichal], [issue/905] [pull/957]**

    Previously we blocked users from declaring `webpack_config` in their `wrangler.toml`, as it can be relatively confusing due to the nested nature of the workers-site directory. We removed that block, and added a friendly help message when webpack build fails and the user has a custom `webpack_config` declared.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/957]: https://github.com/cloudflare/wrangler/pull/957
    [issue/905]: https://github.com/cloudflare/wrangler/issues/905

  - **Reformat config api-key output - [bradyjoslin], [issue/889] [pull/910]**

    We care a lot about our error output. Now the output from `wrangler config` is consistent between calls with and without the `--api-key` flag.

    [bradyjoslin]: https://github.com/bradyjoslin
    [pull/910]: https://github.com/cloudflare/wrangler/pull/910
    [issue/889]: https://github.com/cloudflare/wrangler/issues/889

  - **Improve error message for `wrangler init --site` when wrangler.toml already exists - [ashleygwilliams], [issue/648] [pull/931]**

    `wrangler init` generally expects that you don't already have a `wrangler.toml` present; however it is common that users want to add static site functionality to their existing wrangler project and will try using `wrangler init` to do so. Rather than simply complaining that the toml already exists, now we add the `workers-site` directory to the project, and print out the suggested configuration to add to `wrangler.toml`. Much nicer!

    [ashleygwilliams]: https://github.com/ashleygwilliams
    [pull/931]: https://github.com/cloudflare/wrangler/pull/931
    [issue/648]: https://github.com/cloudflare/wrangler/issues/648

  - **Add a helpful error message on authentication error - [EverlastingBugstopper], [issue/492] [pull/932]**

    Previously, when `wrangler publish` ran into authentication errors, the API result would just print to the screen. Now, it prints a helpful hint to users to re-run `wrangler config` to fix the error.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/932]: https://github.com/cloudflare/wrangler/pull/932
    [issue/492]: https://github.com/cloudflare/wrangler/issues/492

  - **Provide helpful error when user accidentally puts kv-namespace config under `[site]` - [gabbifish], [issue/798] [pull/937]**

    TOML formatting can be tricky, specifically tables, so it is common for users unfamiliar with the format to accidentally nest attributes inside tables without intending it. In this case, if a user adds a kv-namespaces entry to the bottom of a toml with [site] configuration already declared, it is parsed as a part of the [site] table, rather than as a top-level key. The error output from this is not super helpful, as it just says "unknown field `kv-namespaces`" which isn't precisely correct.

    This PR detects when this error occurs and provides a help suggestion to put kv-namespaces ABOVE the [site] table entry to fix the problem.

    [gabbifish]: https://github.com/gabbifish
    [pull/937]: https://github.com/cloudflare/wrangler/pull/937
    [issue/798]: https://github.com/cloudflare/wrangler/issues/798

- ### Fixes

  - **Don't install `wasm-pack` for `webpack` type projects - [EverlastingBugstopper], [issue/745] [pull/849]**

    You may have noticed that Wrangler installs `wasm-pack` for your `webpack` projects, which may seem strange since it's the tool we use to build Rust projects. The reason for this is because you can _also_ build Rust using `wasm-pack` and `webpack` in tandem if you use the [`wasm-pack-plugin`](https://github.com/wasm-tool/wasm-pack-plugin). This plugin recently added support for handling the installation of `wasm-pack` which means Wrangler no longer needs to handle those installs.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/849]: https://github.com/cloudflare/wrangler/pull/849
    [issue/745]: https://github.com/cloudflare/wrangler/issues/745

- ### Maintenance

  - **Make Azure use latest `rustc` - [EverlastingBugstopper], [issue/887] [pull/893]**

    Updates our CI to update the rust toolchain to the latest stable version after installation.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/893]: https://github.com/cloudflare/wrangler/pull/893
    [issue/887]: https://github.com/cloudflare/wrangler/issues/887

  - **Fix nightly builds - [EverlastingBugstopper], [pull/895], [pull/898]**

    Now we confirm Wrangler builds against nightly Rust releases!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/895]: https://github.com/cloudflare/wrangler/pull/895
    [pull/898]: https://github.com/cloudflare/wrangler/pull/898

  - **Fix compiler warnings on windows - [uma0317], [issue/800] [pull/919]**

    We build Wrangler for Mac OSX, Linux, and Windows, and each of these environments has slightly different needs at compile time. In this case, community contributor [uma0317] added configuration that eliminated unused imports for Windows at compile time.

    [uma0317]: https://github.com/uma0317
    [pull/919]: https://github.com/cloudflare/wrangler/pull/919
    [issue/800]: https://github.com/cloudflare/wrangler/issues/800

  - **Remove deprecated kv-namespace config check - [ashleymichal], [pull/929]**

    Back in 1.1.0, we introduced more robust support for adding KV namespaces to your project. It was a breaking change for users who were still using our first pass at configuration for this in their toml, so we added a friendly error message telling them how to update their `wrangler.toml`. At this point, all of our users have safely transitioned onto the new syntax, and so we removed the warning; any lingering use of the old syntax will be met with a parse error instead.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/929]: https://github.com/cloudflare/wrangler/pull/929

  - **Use binary-install for npm - [EverlastingBugstopper], [pull/862]**

    This extracts a lot of the logic in Wrangler's installer to an external package, [binary-install], which we will also use for installing wasm-pack on webpack project builds. Switching to this package also has the added benefit of cleaning up the downloaded binary on `npm uninstall -g @cloudflare/wrangler`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/862]: https://github.com/cloudflare/wrangler/pull/862
    [binary-install]: http://npmjs.org/package/binary-install

## 🎰 1.6.0

- ### Features

  - **_BREAKING CHANGE_: Require the `webpack_config` field in `wrangler.toml` to build with a custom configuration - [EverlastingBugstopper], [issue/296] [pull/847]**

    Wrangler will no longer use a `webpack.config.js` at the root of your project to build your worker. If you would like to continue using a custom build configuration, you will need to specify the `webpack_config` field in your `wrangler.toml` like so:

    ```toml
    name = "my-worker"
    workers_dev = true
    account_id = "01234567890987654321234567890"
    webpack_config = "webpack.config.js"
    ```

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/847]: https://github.com/cloudflare/wrangler/pull/847
    [issue/296]: https://github.com/cloudflare/wrangler/issues/296

  - **API Token Support - [gabbifish]/[ashleymichal], [issue/354] [pull/471]/[pull/879]**

    Wrangler can now be configured with API Tokens!

    Don't worry, current configurations with an email address and a Global API Key will continue to work, but we highly recommend that you switch to API Tokens as they are a much more secure authentication method.

    If you want to use API tokens, create an API token from the "Edit Cloudflare Workers" API token template [here](https://dash.cloudflare.com/profile/api-tokens), and copy/paste it in the `wrangler config` prompt. Alternatively, you can set the `CF_API_TOKEN` environment variable.

    [gabbifish]: https://github.com/gabbifish
    [ashleymichal]: https://github.com/ashleymichal
    [pull/471]: https://github.com/cloudflare/wrangler/pull/471
    [pull/879]: https://github.com/cloudflare/wrangler/pull/879
    [issue/354]: https://github.com/cloudflare/wrangler/issues/354

  - **Add the ability to preview without opening the browser - [EverlastingBugstopper], [issue/256] [pull/816]**

    `wrangler preview` can now be called with a `--headless` flag that will not open the browser.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/816]: https://github.com/cloudflare/wrangler/pull/816
    [issue/256]: https://github.com/cloudflare/wrangler/issues/256

  - **Check for valid credentials when running `wrangler config` - [gabbifish], [issue/439] [pull/842]**

    Before this version of Wrangler, `wrangler config` would allow any input string to be passed for your user details. Now, Wrangler validates that the credentials will work with Cloudflare's API.

    [gabbifish]: https://github.com/gabbifish
    [pull/842]: https://github.com/cloudflare/wrangler/pull/842
    [issue/439]: https://github.com/cloudflare/wrangler/issues/439

  - **Add a warning when publishing a Workers Site to a route without a trailing asterisk - [EverlastingBugstopper], [issue/814] [pull/839]**

    When publishing a Workers Site to your own domain, it's important that the Worker code runs on every path on your domain. This isn't particularly clear, so now when attempting to publish a Workers Site to a route without a trailing asterisk, Wrangler will print a warning message.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/839]: https://github.com/cloudflare/wrangler/pull/839
    [issue/814]: https://github.com/cloudflare/wrangler/issues/814

  - **Better error message for publishing to a duplicate route - [pradovic], [issue/519] [pull/813]**

    When publishing to a route that is associated with another worker, Wrangler now prints a more actionable error message.

    [pradovic]: https://github.com/pradovic
    [pull/813]: https://github.com/cloudflare/wrangler/pull/813
    [issue/519]: https://github.com/cloudflare/wrangler/issues/519

  - **Better error message when webpack fails - [ashleymichal], [issue/428] [pull/837]**

    Wrangler now recommends running `npm install` as a possible remedy for failed webpack builds.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/837]: https://github.com/cloudflare/wrangler/pull/837
    [issue/428]: https://github.com/cloudflare/wrangler/issues/428

- ### Fixes

  - **Properly handle errors when running Wrangler as a global npm package - [jaredmcdonald], [issue/848] [pull/857]**

    [jaredmcdonald]: https://github.com/jaredmcdonald
    [pull/857]: https://github.com/cloudflare/wrangler/pull/857
    [issue/848]: https://github.com/cloudflare/wrangler/issues/848

  - **Clean up temporary build files - [EverlastingBugstopper], [pull/853]**

    When building a script, Wrangler creates a temporary file. Old versions of Wrangler were quite messy about it, but now it cleans up after itself.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/853]: https://github.com/cloudflare/wrangler/pull/853

  - **Fix the help text for `wrangler generate` - [EverlastingBugstopper], [pull/830]**

    The default value for a template is now a complete and valid URL instead of a sample project name.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/830]: https://github.com/cloudflare/wrangler/pull/830

  - **Remove --version on subcommands - [EverlastingBugstopper], [issue/791] [pull/829]**

    Each subcommand in Wrangler used to take a `--version` argument which would print the name of the subcommand. For instance, `wrangler publish --version` would print `wrangler-publish`. This wasn't super helpful, so we've removed that functionality.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/829]: https://github.com/cloudflare/wrangler/pull/829
    [issue/791]: https://github.com/cloudflare/wrangler/issues/791

  - **Fix a broken link in the README - [victoriabernard92], [pull/838]**

    [victoriabernard92]: https://github.com/victoriabernard92
    [pull/838]: https://github.com/cloudflare/wrangler/pull/838

- ### Maintenance

  - **Create fixtures programmatically - [EverlastingBugstopper], [pull/854]**

    Wrangler's test suite relied on a large number of fixtures that it read in from the file system. Now, it writes the test fixtures itself and does not rely on reading fixtures from the file system.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/854]: https://github.com/cloudflare/wrangler/pull/854

  - **Clean up Workers Sites logic - [ashleymichal], [issue/622] [issue/643] [pull/851]**

    [ashleymichal]: https://github.com/ashleymichal
    [pull/851]: https://github.com/cloudflare/wrangler/pull/851
    [issue/622]: https://github.com/cloudflare/wrangler/issues/622
    [issue/643]: https://github.com/cloudflare/wrangler/issues/643

  - **Call cloudflare-rs from https.rs - [gabbifish], [pull/841]**

    We've refactored some of our API client code in order to make way for some future improvements.

    [gabbifish]: https://github.com/gabbifish
    [pull/841]: https://github.com/cloudflare/wrangler/pull/841

  - **Audit code comments - [EverlastingBugstopper], [pull/846]**

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/846]: https://github.com/cloudflare/wrangler/pull/846

  - **Update the author of the npm package - [EverlastingBugstopper], [pull/836]**

    The author of the npm package is now [wrangler@cloudflare.com](mailto:wrangler@cloudflare.com)

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/836]: https://github.com/cloudflare/wrangler/pull/836

  - **Remove unused code warnings when running tests - [pradovic], [issue/818] [pull/832]**

    Due to the way the Rust compiler works, some of our test code appeared to be unused, even though it wasn't really. After making a couple of modules public, there are no more warnings.

    [pradovic]: https://github.com/pradovic
    [pull/832]: https://github.com/cloudflare/wrangler/pull/832
    [issue/818]: https://github.com/cloudflare/wrangler/issues/818

  - **Use the same binding name for Rust and webpack wasm modules - [ashleymichal], [pull/822]**

    [ashleymichal]: https://github.com/ashleymichal
    [pull/822]: https://github.com/cloudflare/wrangler/pull/822

  - **Move the code for each subcommand to its own directory - [EverlastingBugstopper], [pull/831]**

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/831]: https://github.com/cloudflare/wrangler/pull/831

  - **Refactor upload forms - [ashleymichal], [pull/826]**

    We've separated some tangled logic regarding the form Wrangler POSTs to the Cloudflare v4 API.

    [ashleymichal]: https://github.com/ashleymichal
    [pull/826]: https://github.com/cloudflare/wrangler/pull/826

  - **Pull npm version from package.json - [EverlastingBugstopper], [issue/812] [pull/817]**

    Wrangler's npm installer version now only needs updating in the package.json instead of both the package.json and the source code.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/817]: https://github.com/cloudflare/wrangler/pull/817
    [issue/812]: https://github.com/cloudflare/wrangler/issues/812

- ### Documentation

  - **Move Wrangler docs from READMEs to the Cloudflare Workers documentation site - [victoriabernard92], [pull/823]**

    Wrangler has outgrown the README as primary documentation paradigm, and we've moved its documentation to the [Cloudflare Workers documentation site](https://developers.cloudflare.com/workers/tooling/wrangler/).

    [victoriabernard92]: https://github.com/victoriabernard92
    [pull/823]: https://github.com/cloudflare/wrangler/pull/823

  - **Update the demo gif in the README - [EverlastingBugstopper], [issue/843] [pull/868]**

    The demo gif at the top of the README now accurately reflects the behavior of the latest Wrangler release.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/868]: https://github.com/cloudflare/wrangler/pull/868
    [issue/843]: https://github.com/cloudflare/wrangler/issues/843

## 👻 1.5.0

- ### Features

  - **Deprecate `wrangler publish --release` - [EverlastingBugstopper], [issue/538] [pull/751]**

    `wrangler publish --release` is now simply an alias of `wrangler publish`. This is related to the introduction of [environments](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md) made in [1.3.1](#-131) and is intended to reduce confusion surrounding deploy targets. Previously, `wrangler publish --release` would deploy to a route on your own domain, and `wrangler publish` would deploy to your workers.dev subdomain. This was a confusing API, and we now require individual environments to have either `workers_dev = true` **or** both a `route` and `zone_id` in each section of `wrangler.toml`. This makes it very clear where your Workers code is being deployed. If you're not using `wrangler publish --release` but you added `workers_dev = false` to the top level of your `wrangler.toml` because Wrangler warned you to - you can now safely remove it! If you **are** using `wrangler publish --release`, know that it is functionally the same as `wrangler publish`. If you want to deploy to workers.dev and also a route on your own domain, you will need to set up multiple environments.

    [issue/538]: https://github.com/cloudflare/wrangler/issues/538
    [pull/751]: https://github.com/cloudflare/wrangler/pull/751

  - **Deprecate `private` field in `wrangler.toml` - [stevenfranks], [issue/782] [pull/782]**

    In a related note, the `private` field no longer functions in `wrangler.toml`. The original intent behind this field was to allow "publishing" without "activating". Unfortunately this led to a lot of undefined behavior if the value was switched from `true` to `false` in between a `wrangler publish` command and a `wrangler publish --release` command and vice versa. With the removal of `wrangler publish --release`, we are also removing the `private` field. If your `wrangler.toml` files contain a value for private, you can remove it!

    [stevenfranks]: https://github.com/stevenfranks
    [pull/782]: https://github.com/cloudflare/wrangler/pull/782
    [issue/782]: https://github.com/cloudflare/wrangler/issues/782

  - **Include/exclude static assets in a Workers Sites project - [gabbifish], [issue/716] [pull/760]**

    Your `wrangler.toml` has two new optional fields: `include` and `exclude`. These fields give you more granular control over what files are uploaded to Workers KV. This behavior mirrors Cargo's [include/exclude](https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields-optional) functionality. Further documentation for this feature is available [here](https://developers.cloudflare.com/workers/sites/ignore-assets/).

    [issue/716]: https://github.com/cloudflare/wrangler/issues/716
    [pull/760]: https://github.com/cloudflare/wrangler/pull/760

  - **A more robust `wrangler generate` - [EverlastingBugstopper], [issue/315] [pull/759]**

    `wrangler generate` is now much smarter about `wrangler.toml` files. Previously, `wrangler generate` would simply create the same configuration for every project, and it would ignore any `wrangler.toml` that was committed to the template. This means much less guesswork when using `wrangler generate` with existing Workers projects.

    [issue/315]: https://github.com/cloudflare/wrangler/issues/315
    [pull/759]: https://github.com/cloudflare/wrangler/pull/759

  - **Add the ability to check if you've already registered a workers.dev subdomain - [gusvargas], [issue/701] [pull/747]**

    You can now run `wrangler subdomain` without any arguments to see if you have registered a [workers.dev](https://workers.dev) subdomain.

    ```sh
    $ wrangler subdomain
    💁  foo.workers.dev
    ```

    [gusvargas]: https://github.com/gusvargas
    [pull/747]: https://github.com/cloudflare/wrangler/pull/747
    [issue/701]: https://github.com/cloudflare/wrangler/issues/701

  - **Add `--verbose` flag to `wrangler publish` and `wrangler preview` - [gabbifish], [issue/657] [pull/790]**

    You can now run `wrangler publish --verbose` and `wrangler preview --verbose` on a Workers Sites project to view all of the files that are being uploaded to Workers KV.

    ```sh
    $ wrangler publish --verbose
    🌀  Using namespace for Workers Site "__example-workers_sites_assets"
    💁  Preparing to upload updated files...
    🌀  Preparing ./public/favicon.ico
    🌀  Preparing ./public/index.html
    🌀  Preparing ./public/404.html
    🌀  Preparing ./public/img/404-wrangler-ferris.gif
    🌀  Preparing ./public/img/200-wrangler-ferris.gif
    ✨  Success
    ✨  Built successfully, built project size is 11 KiB.
    ✨  Successfully published your script to https://test.example.workers.dev
    ```

    [issue/657]: https://github.com/cloudflare/wrangler/issues/657
    [pull/790]: https://github.com/cloudflare/wrangler/pull/790

  - **Disallow `node_modules` as a bucket for Workers Sites - [gabbifish], [issue/723] [pull/792]**

    `node_modules` is no longer allowed to be a bucket for Workers Sites. It is notoriously very large and if it were specified as a bucket it would probably be a very expensive mistake.

    [issue/723]: https://github.com/cloudflare/wrangler/pull/792
    [pull/792]: https://github.com/cloudflare/wrangler/issues/723

  - **Allow installs to utilize Wrangler binaries via a caching proxy instead of GitHub directly - [gabbifish], [pull/797]**

    To avoid dependency on one external service, GitHub, we enabled a cache proxy (using Workers!) for installations of Wrangler.

    [gabbifish]: https://github.com/cloudflare/wrangler/pull/797

  - **Provide a better error message when using an unverified email address - [ashleygwilliams], [issue/320] [pull/795]**

    The Cloudflare API refuses to play nice with unverified email addresses (we don't like spam!), and now when this happens, Wrangler gives an actionable error message.

    [issue/320]: https://github.com/cloudflare/wrangler/issues/320
    [pull/795]: https://github.com/cloudflare/wrangler/pull/795

- ### Fixes

  - **Fix Rust live preview - [gabbifish], [issue/618] [pull/699]**

    If you use Wrangler to develop Rust Workers, you may have noticed that live preview (`wrangler preview --watch`) was not working with your project. Not to worry though, we cracked down on this bug with an (oxidized) iron fist! Wrangler now has cross-platform support for live previewing Rust Workers.

    [issue/618]: https://github.com/cloudflare/wrangler/issues/618
    [pull/699]: https://github.com/cloudflare/wrangler/pull/699

  - **Minimize timeout errors for bulk uploads - [gabbifish], [issue/746] [pull/757]**

    Sometimes Wrangler would make API calls to Workers KV that would timeout if there were too many files. We've increased the amount of time Wrangler will wait around for the API operations to complete.

    [issue/746]: https://github.com/cloudflare/wrangler/issues/746
    [pull/757]: https://github.com/cloudflare/wrangler/pull/757

  - **Print readable error message when external commands fail - [EverlastingBugstopper], [pull/799]**

    Wrangler depends on a few external applications, and sometimes the calls to them fail! When this happens, Wrangler would tell you the command it tried to run, but it included a bunch of quotes. This change removes those quotes so the command is easily readable and can be copy/pasted.

    [pull/799]: https://github.com/cloudflare/wrangler/pull/799

  - **Disallow `wrangler generate --site` with a template argument - [EverlastingBugstopper], [issue/776] [pull/789]**

    In Wrangler 1.4.0, we introduced [Workers Sites](https://developers.cloudflare.com/workers/sites/), which included the ability to run `wrangler generate --site` which would use our site template behind the scenes. Confusingly, you could also pass a template to this command: `wrangler generate my-site https://github.com/example/worker-site --site`, which would not behave as expected. This specific usage will now correctly output an error.

    [issue/776]: https://github.com/cloudflare/wrangler/issues/776
    [pull/789]: https://github.com/cloudflare/wrangler/pull/789

- ### Maintenance

  - **Begin refactoring test suite - [ashleymichal], [pull/787]**

    We're constantly shipping features in Wrangler, and with more features comes a larger codebase. As a codebase expands, it goes through some growing pains. This release includes some improvements to the internal organization of Wrangler's codebase, and is intended to make our lives and our contributors' lives easier moving forward.

    - Moved all "fixture" helper functions to "utils" module to share between build/preview tests

    - Removed "metadata_wasm.json" from `simple_rust` fixture

    - Extracted all module declarations in `main.rs` to `lib.rs` to allow tests to import with `use wrangler::foo`

    - Split `target/mod.rs` into one file per struct

    - Cleaned up KV Namespace mod system

    - Use `log::info!` instead of `info!` in `main.rs`

    [pull/787]: https://github.com/cloudflare/wrangler/pull/787

  - **Refactor GlobalUser to be passed as a reference consistently - [gabbifish], [pull/749]**

    [pull/749]: https://github.com/cloudflare/wrangler/pull/749

  - **Remove internal link from CONTRIBUTING.md - [adaptive], [pull/784]**

    [adaptive]: https://github.com/adaptive
    [pull/784]: https://github.com/cloudflare/wrangler/pull/784

  - **Fix some [Clippy](https://github.com/rust-lang/rust-clippy) warnings - [EverlastingBugstopper], [pull/793]**

    [pull/793]: https://github.com/cloudflare/wrangler/pull/793

  - **Clean up leftover directories created by tests - [ashleymichal], [pull/785]**

    [pull/785]: https://github.com/cloudflare/wrangler/pull/785

  - **Refactor subdomain module - [EverlastingBugstopper], [issue/758] [pull/764]**

    [issue/758]: https://github.com/cloudflare/wrangler/issues/758
    [pull/764]: https://github.com/cloudflare/wrangler/pull/764

  - **Fix README markdown misrender - [dottorblaster], [pull/763]**

    [dottorblaster]: https://github.com/dottorblaster
    [pull/763]: https://github.com/cloudflare/wrangler/pull/763

  - **Remove duplicate Environments subheader from README - [bradyjoslin], [pull/766]**

    [bradyjoslin]: https://github.com/bradyjoslin
    [pull/766]: https://github.com/cloudflare/wrangler/pull/766

  - **Change Crate author to the Workers Developer Experience team - [ashleygwilliams], [pull/752]**

    [pull/752]: https://github.com/cloudflare/wrangler/pull/752

## 🎂 1.4.0

- ### Features

  - **Workers Sites - [pull/509]**

    Wrangler 1.4.0 includes supports for **Workers Sites**, enabling developers to deploy static applications directly to Workers. Workers Sites is perfect for frontend frameworks like [React](https://reactjs.org) and [Vue](https://vuejs.org/), as well as static site generators like [Hugo](https://gohugo.io/) and [Gatsby](https://gohugo.io/).

    Workers Sites is a feature exclusive to Wrangler, and combines a great developer experience with excellent performance. The `--site` flag has been added to `wrangler init` and `wrangler generate` to use Workers Sites with new and existing projects:

    ```sh
    # Add Workers Sites to an existing project
    $ wrangler init --site

    # Start a new Workers Sites project
    $ wrangler generate --site
    ```

    If you're ready to get started using Workers Sites, we've written guides for the various routes you might take with your project:

    - [Create a new project from scratch](https://developers.cloudflare.com/workers/sites/start-from-scratch)
    - [Deploy a pre-existing static site project](https://developers.cloudflare.com/workers/sites/start-from-existing)
    - [Add static assets to a pre-existing Workers project](https://developers.cloudflare.com/workers/sites/start-from-worker)

    For more details on how Workers Sites works with Wrangler, check out [the documentation](https://developers.cloudflare.com/workers/sites/reference). We also have a brand new [tutorial](https://developers.cloudflare.com/workers/tutorials/deploy-a-react-app) to help you learn the Workers Sites workflow, by deploying a React application!

    Workers Sites has been a heroic effort by the entire Workers Developer Experience team, comprising of Wrangler updates, new [project templates](https://github.com/cloudflare/worker-sites-template), and [open-source packages](https://github.com/cloudflare/kv-asset-handler). We're super excited about the future that Workers Sites represents, where static sites and serverless functions can work together to build powerful, cutting-edge applications.

    Make sure to try out Workers Sites to build your next app! 🎉🎉🎉

    [pull/509]: https://github.com/cloudflare/wrangler/pull/509

  - **Download release from our proxy rather than GitHub - [zackbloom], [pull/692]**

    [zackbloom]: https://github.com/zackbloom
    [pull/692]: https://github.com/cloudflare/wrangler/pull/692

  - **Add validation for workers names in wrangler init and wrangler generate - [gabbifish], [issue/470][pull/686]**

    There are a number of requirements around _what_ your Workers script is named – previously, Wrangler would opaquely fail and not indicate that your script name was invalid: this PR updates the `init` and `generate` commands to validate the potential name before continuing to create a new project.

    [gabbifish]: https://github.com/gabbifish
    [issue/470]: https://github.com/cloudflare/wrangler/issues/470
    [pull/686]: https://github.com/cloudflare/wrangler/pull/686

  - **Ensure KV subcommands check for presence of required fields in wrangler.toml - [gabbifish], [issue/607] [pull/665]**

    There's a number of commands in `wrangler` that require a properly configured `wrangler.toml` file: instead of failing, this PR ensures that these commands now check your configuration file before attempting any action. Hooray for clarity! 😇

    [gabbifish]: https://github.com/gabbifish
    [pull/665]: https://github.com/cloudflare/wrangler/pull/665

  - **Show size when a KV upload error occurs - [stevenfranks], [issue/650] [pull/651]**

    Previously, when uploading a file to Workers KV from Wrangler, the error output didn't indicate the size of the file that was being uploaded. This PR improves the output of that message by showing both the file size and the maximum file size that can be uploaded to Workers KV.

    [stevenfranks]: https://github.com/stevenfranks
    [issue/650]: https://github.com/cloudflare/wrangler/issues/650
    [pull/651]: https://github.com/cloudflare/wrangler/pull/651

  - **Better file support for kv:put - [phayes], [pull/633]**

    The `kv:put` command, introduced in Wrangler 1.3.1, has been improved to support uploading non-UT8 files. In addition, the command now streams files directly to the Cloudflare API when uploading, instead of buffering them in memory during the upload process.

    [phayes]: https://github.com/phayes
    [pull/633]: https://github.com/cloudflare/wrangler/pull/633

- ### Fixes

  **Ensure we install and cache the latest version of cargo-generate and wasm-pack if user has an outdated cargo installed version - [EverlastingBugstopper], [issue/666] [pull/726]**

    Wrangler orchestrates a few other tools under the hood, notably [`wasm-pack`](https://github.com/rustwasm/wasm-pack) and [`cargo-generate`](https://github.com/ashleygwilliams/cargo-generate). We use a library called [`binary-install`](https://github.com/rustwasm/binary-install) to fetch and cache binaries we download. However, to avoid downloading unnecessarily, we first check if the user has a copy locally on their machine that they had `cargo install`'d. We had a bug where in this logic branch, we *didn't* check that the local version was the most up-to-date version. This meant that users who had an older installed version may run into errors when wrangler expected to use features of a newer version of that tool. This PR adds the logic to check for the version and will install and cache a newer version for wrangler to use (leaving your local version as is!).

  [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
  [issue/666]: https://github.com/cloudflare/wrnagler/issues/666
  [pull/726]: https://github.com/cloudflare/wrangler/pull/726

  - **Remove link to 000000000000000000.cloudflareworkers.com - [EverlastingBugstopper], [pull]**

    Have you ever run `wrangler preview` in your project and wondered why the URL to preview your application is `000000000000000000.cloudflareworkers.com`? The writer of this CHANGELOG finds it confusing, too: this PR removes that line, making it easier to parse the output from `wrangler preview`.

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull]: https://github.com/cloudflare/wrangler/pull/698

  - **Make install actually fail if the release can't be downloaded - [zackbloom], [pull/672]**

    When Wrangler's installation shim attempts to install Wrangler on your machine, there's the possibility that the installation can fail – instead of failing silently, the installation shim now properly throws an error, allowing us to better diagnose installation failures.

    [zackbloom]: https://github.com/zackbloom
    [pull/672]: https://github.com/cloudflare/wrangler/pull/672

- ### Maintenance

  - **KV command error output improvements - [gabbifish], [issue/608] [pull/613]**

    The Wrangler team is always on the quest for perfect error messages. In pursuit of that goal, we've improved how errors in the `wrangler kv` subcommands output to your terminal. 😎

    [gabbifish]: https://github.com/gabbifish
    [issue/608]: https://github.com/cloudflare/wrangler/pull/608
    [pull/613]: https://github.com/cloudflare/wrangler/pull/613

  - **Added missing word to whoami response - [stevenfranks], [pull/695]**

    Clear writing is good! It's hard to write clearly when words are missing in a sentence. This PR corrects the output of `wrangler whoami` to add a missing word, making this command easier to read. 🤔

    [stevenfranks]: https://github.com/stevenfranks
    [pull/695]: https://github.com/cloudflare/wrangler/pull/695

- ### Documentation

  - **Webpack documentation - [EverlastingBugstopper], [issue/721] [pull/724]**

    For our default build type, aptly named "webpack", Wrangler uses webpack under the hood to bundle all of your assets. We hadn't documented how we do that, what our default config is, and how you can specify your own custom webpack config if you'd like. We have those docs now, so [check them out]!

    [check them out]: https://github.com/cloudflare/wrangler/blob/master/docs/content/webpack.md
    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [issue/721]: https://github.com/cloudflare/wrangler/issues/721
    [pull/724]: https://github.com/cloudflare/wrangler/pull/724

## 🐛 1.3.1

- ### Features

  - **Environments - [EverlastingBugstopper], [issue/385][pull/386]**

    Wrangler 1.3.1 includes supports for **environments**, allowing developers to deploy Workers projects to multiple places. For instance, an application can be deployed to a production URL _and_ a staging URL, without having to juggle multiple configuration files.

    To use environments, you can now pass in `[env.$env_name]` properties in your `wrangler.toml`. Here's an example:

    ```toml
    type = "webpack"
    name = "my-worker-dev"
    account_id = "12345678901234567890"
    zone_id = "09876543210987654321"
    workers_dev = false

    [env.staging]
    name = "my-worker-staging"
    route = "staging.example.com/*"

    [env.production]
    name = "my-worker"
    route = "example.com/*"
    ```

    With multiple environments defined, `wrangler build`, `wrangler preview`, and `wrangler publish` now accept a `--env` flag to indicate what environment you'd like to use, for instance, `wrangler publish --env production`.

    To support developers transitioning to environments, we've written documentation for the feature, including further information about deprecations and advanced usage. [Check out the documentation here!](https://github.com/cloudflare/wrangler/blob/master/docs/content/environments.md)

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [issue/385]: https://github.com/cloudflare/wrangler/issues/385
    [pull/386]: https://github.com/cloudflare/wrangler/pull/386

  - **KV commands - [ashleymichal], [gabbifish], [issue/339][pull/405]**

    Wrangler 1.3.1 includes commands for managing and updating [Workers KV](https://www.cloudflare.com/products/workers-kv/) namespaces, keys, and values directly from the CLI.

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

    The KV subcommands in Wrangler 1.3.1 make it super easy to comfortably query and manage your Workers KV data without ever having to leave the command-line. For more information on the available commands and their usage, see [the documentation](https://github.com/cloudflare/wrangler/blob/master/docs/content/kv_commands.md). 🤯

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

  - **Different emojis for different commands - [EverlastingBugstopper], [pull/605]**

    KV subcommands would return the same emoji value in `--help` output. This PR updates the command-line output to use different emoji, making the output easier to read!

    [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
    [pull/605]: https://github.com/cloudflare/wrangler/pull/605

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

    This PR cleans up some incorrectly named tests and adds fixtures to support testing new functionality in 1.3.1, such as environments. ✨

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

    Previously the NPM installer for wrangler would always pull the most recent release from GitHub releases, and the installer did not increase version numbers when Wrangler did. Many users found this confusing. Now the installer will increment versions along with Wrangler releases, and point at specific versions rather than the most recent one at the time of installation.

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

  - **Update issue templates to fit GitHub's data model - [EverlastingBugstopper], [pull/387]**

    Our previous issue templates were not picked up by GitHub's user interface. This PR updates the templates to fit the accepted data model, and adds some style tweaks to make the templates easier to use.

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

    We've made the `wrangler config` command interactive – the previous version of the command, `wrangler config $email $apiKey`, would be captured by your terminal's history, often exposing that information in a `~/.bash_history` or a similar file. The new version of `wrangler config` will prompt you for your `email` and `api_key` via `stdin`.

    In addition, this PR also adds support for a `WRANGLER_HOME` environment variable, which will be the location for Wrangler's "home" directory, if you need to customize where Wrangler saves its configuration information.

    [xtuc]: https://github.com/xtuc
    [pull/329]: https://github.com/cloudflare/wrangler/pull/239

* ### Features

  - **Support KV Namespace Configuration - [ashleymichal], [pull/334], add check + error message for pre 1.1.0 kv namespace format - [xortive], [pull/369]**

    Wrangler now supports using [Workers KV][kv] namespaces in your project! To start using KV with your projects, create a namespace in the Cloudflare Dashboard, and the namespace information to your `wrangler.toml` configuration file. The `kv-namespaces` key requires setting a `binding` (the representation of your namespace in your code) and `id`, the namespace ID:

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

    It's super easy to grab a workers.dev subdomain using the `subdomain` command in `wrangler` – so easy, in fact, that many people were trying to use it without even having a Cloudflare account! `wrangler` now warns users when they attempt to add a subdomain without configuring their `account_id` in `wrangler.toml`, as well as when you've already registered a subdomain, or if the subdomain you're trying to register has already been claimed.

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

    `wrangler` now returns an error during the build process if you use a webpack configuration with more than one export – `wrangler` needs to have a single known export from webpack to know what to build!

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

  - **Terminal messaging abstraction - [ashleymichal], [issue/219][pull/263]**

    We've made improvements to Wrangler's terminal output functionality, with support for various log levels and implementations in Wrangler's API for easily using the log levels in future development.

    The new terminal output functionality can be used by importing the `terminal::message` crate:

    ```rust
    use crate::terminal::message;

    message::info("Building project") // "💁‍ Building project"
    message::success("Your project has been deployed!") // "✨ Your project has been deployed!"

    // Other available functions:
    // message::warn, message::user_error, message::working, message::preview
    ```

    [ashleymichal]: https://github.com/ashleymichal
    [issue/219]: https://github.com/cloudflare/wrangler/issues/219
    [pull/263]: https://github.com/cloudflare/wrangler/pull/263

  - **Remove pre-push hooks - [EverlastingBugstopper], [pull/308]**

    Previous versions of Wrangler included pre-push hooks to ensure that code was linted before being pushed up to Git. This hook made it difficult to manage in-progress work, so the hooks have been removed.

    [pull/308]: https://github.com/cloudflare/wrangler/pull/308

  - **Use serde for metadata - [xtuc], [pull/285]**

    This change adds proper construction of the worker metadata, previously, it was an error-prone string.

    [pull/285]: https://github.com/cloudflare/wrangler/pull/285

  - **Refactor: Conditional per command in main - [ashleymichal], [pull/279]**

    The `src/main.rs` file in Wrangler has been rewritten so that the layout of the file is easier to read.

    [ashleymichal]: https://github.com/ashleymichal
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