# Changelog

## 💥 0.1.1

  - ### 🤕 Fixes

    - **Fix `publish` and `preview` bug for projects with a `-` - [jaysonsantos], [issue/36] [pull/38]**

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

    - **Install instructions with OpenSSL troubleshooting notes - [AustinCorridor], [issue/35] [pull/43]**

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
