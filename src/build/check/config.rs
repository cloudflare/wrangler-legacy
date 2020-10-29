use bytesize::ByteSize;
use swc_ecma_parser::{EsConfig, Syntax};
use wasmparser::WasmFeatures;

/// The parseable syntax we allow, as dictated by V8 stable.
/// Check the [SWC docs](https://swc.rs/rustdoc/swc_ecma_parser/struct.EsConfig.html) for the available options.
#[doc(inline)]
pub const V8_SUPPORTED_JS_FEATURES: Syntax = Syntax::Es(EsConfig {
    // sir, this is a wendy's...
    jsx: false,
    // https://v8.dev/blog/v8-release-75#numeric-separators
    num_sep: true,
    // https://v8.dev/features/class-fields#private-class-fields
    // applies to both props and methods i think
    class_private_props: true,
    class_private_methods: true,
    // https://v8.dev/features/class-fields#public-class-fields
    class_props: true,
    // https://chromium.googlesource.com/v8/v8/+/3.0.12.1/test/mjsunit/function-bind.js
    fn_bind: true,
    // AFAIK this is still...due to be presented? it is now october but
    // applies to both decorators and decorators_before_export
    // rfc: https://github.com/tc39/proposal-decorators
    // V8 team's feedback: https://docs.google.com/document/d/1GMp938qlmJlGkBZp6AerL-ewL1MWUDU8QzHBiNvs3MM/edit
    decorators: false,
    decorators_before_export: false,
    // https://v8.dev/features/modules
    export_default_from: true,
    // https://v8.dev/features/module-namespace-exports
    export_namespace_from: true,
    // https://v8.dev/features/dynamic-import
    dynamic_import: true,
    // https://v8.dev/features/nullish-coalescing
    nullish_coalescing: true,
    // https://v8.dev/features/optional-chaining
    optional_chaining: true,
    // https://v8.dev/features/modules#import-meta
    import_meta: true,
    // ok so for top-level await, V8 says there's no support in
    // "classic scripts", which i think is how workers are executed.
    // i mean, they're certainly not modules.
    // https://v8.dev/features/top-level-await#new
    top_level_await: false,
    // i literally cannot find a source on this
    // i don't...is it `console.assert?`
    // TODO: ask in the slack about this one
    import_assertions: false,
});

/// The features we allow during our validation of WebAssembly, as per V8 stable.
/// Check the [wasmparser](https://docs.rs/wasmparser/0.63.0/wasmparser/struct.WasmFeatures.html)
/// for more.
#[doc(inline)]
pub const V8_SUPPORTED_WASM_FEATURES: WasmFeatures = WasmFeatures {
    // https://www.chromestatus.com/feature/5166497248837632
    reference_types: false,
    // there's a proposal for module linking here
    // https://github.com/WebAssembly/module-linking
    // but it's different than the "dynamic linking" one the v8 team has talked about here
    // https://v8.dev/blog/webassembly-experimental
    // so i'm not sure what the right answer for this is.
    // either way, i wasn't able to find a chromestatus page for it.
    // based on the name, though, i don't think it matters
    // because Workers only supports one WASM file anyway
    module_linking: false,
    // https://www.chromestatus.com/feature/6533147810332672
    simd: false,
    // https://www.chromestatus.com/feature/5192420329259008
    multi_value: false,
    // https://www.chromestatus.com/feature/5724132452859904
    threads: true,
    // https://www.chromestatus.com/feature/5423405012615168
    tail_call: false,
    // https://www.chromestatus.com/feature/4590306448113664
    bulk_memory: true,
    // TODO: i don't know what this is
    deterministic_only: false,
    // RFC: https://github.com/WebAssembly/multi-memory
    // TODO: I also can't find a chromestatus for this
    multi_memory: false,
    // as far as I can tell, this isn't even in the works yet
    // https://v8.dev/blog/4gb-wasm-memory
    // https://github.com/WebAssembly/memory64
    memory64: false,
};

/// The [maximum file size](https://developers.cloudflare.com/workers/platform/limits#script-size) we allow.
#[doc(inline)]
pub const MAX_FILE_SIZE: ByteSize = ByteSize(1_000_000);
