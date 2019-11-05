pub fn get_lib() -> String {
    return r#"
        extern crate cfg_if;
        extern crate wasm_bindgen;
        
        mod utils;
        
        use cfg_if::cfg_if;
        use wasm_bindgen::prelude::*;
        
        cfg_if! {
            // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
            // allocator.
            if #[cfg(feature = "wee_alloc")] {
                extern crate wee_alloc;
                #[global_allocator]
                static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
            }
        }
        
        #[wasm_bindgen]
        pub fn greet() -> String {
            "Hello, wasm-worker!".to_string()
        }
    "#
    .to_string();
}

pub fn get_utils() -> String {
    return r#"
    use cfg_if::cfg_if;

    cfg_if! {
        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        if #[cfg(feature = "console_error_panic_hook")] {
            extern crate console_error_panic_hook;
            pub use self::console_error_panic_hook::set_once as set_panic_hook;
        } else {
            #[inline]
            pub fn set_panic_hook() {}
        }
    }
    "#
    .to_string();
}

pub fn get_cargo_toml() -> String {
    return r#"
    [package]
    name = "worker"
    version = "0.1.0"
    authors = ["The Wrangler Team <wrangler@cloudflare.com>"]
    edition = "2018"

    [lib]
    crate-type = ["cdylib", "rlib"]

    [features]
    default = ["console_error_panic_hook"]

    [dependencies]
    cfg-if = "0.1.2"
    wasm-bindgen = "0.2"

    # The `console_error_panic_hook` crate provides better debugging of panics by
    # logging them with `console.error`. This is great for development, but requires
    # all the `std::fmt` and `std::panicking` infrastructure, so isn't great for
    # code size when deploying.
    console_error_panic_hook = { version = "0.1.1", optional = true }

    # `wee_alloc` is a tiny allocator for wasm that is only ~1K in code size
    # compared to the default allocator's ~10K. It is slower than the default
    # allocator, however.
    #
    # Unfortunately, `wee_alloc` requires nightly Rust when targeting wasm for now.
    wee_alloc = { version = "0.4.2", optional = true }

    [dev-dependencies]
    wasm-bindgen-test = "0.2"

    [profile.release]
    # Tell `rustc` to optimize for small code size.
    opt-level = "s" 
    "#
    .to_string();
}
