use std::path::PathBuf;

use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_ast::{Script, Stmt};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax};

use super::check_file_size;

// // i'm not sure whether it's better to do a pointer to an array of &str, or specify the length.
// // my gut feeling is that specifying the length is better since the lengths are known at compile time
// const UNAVAILABLE_BUILTINS: [&str; 2] = ["eval", "new Function"];
// const AVAILABLE_BUILTINS: [&str; 5] = ["atob", "btoa", "TextEncoder", "TextDecoder", "URL"];
// const AVAILABLE_WITHIN_REQUEST_CONTEXT: [&str; 5] = [
//     "setInterval",
//     "clearInterval",
//     "setTimeout",
//     "clearTimeout",
//     "fetch",
// ];

// this defines the parseable syntax we allow.
// you can check the SWC docs for the available options.
// features are commented showing why or why not we support them.
// i'm targeting everything supported by V8, because we're not actually
// doing any transpilation, just parsing.
const V8_SUPPORTED_FEATURES: Syntax = Syntax::Es(EsConfig {
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
    // AFAIK this is still...due to be presented? september is basically over but idk.
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

pub fn check_js(js_file: &PathBuf) -> Result<String, failure::Error> {
    // might as well check the file size before doing any AST trickery
    // so we fail fast
    let file_size = check_file_size(js_file)?;

    // parse the JS file into a swc_ecma_ast::Script
    // fails if there are any unrecoverable parser errors
    let parsed = load_js(js_file)?;

    let _ = check_ast_for_forbidden_knowledge(parsed.body)?;

    Ok(format!("worker.js OK! Final size: {}", file_size))
}

fn load_js(js_file: &PathBuf) -> Result<Script, failure::Error> {
    // while i remain unsure what "cm" and "fm" are short for, i sure do see
    // them a lot. "Content Manager" and "File Manager" maybe?
    // TODO lol ask in the slack what "cm" and "fm" are short for
    //
    // Lrc is just aliased to Arc or Rc depending on concurrency.
    // https://github.com/swc-project/swc/blob/master/common/src/sync.rs#L14
    //
    // also, it's worth noting that the SourceMap struct has nothing to do
    // with javascript source maps. it refers to a map of like, the in-memory
    // representation of the javascript as held on to by SWC
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.load_file(js_file)?;

    // should we actually do something about the comments?
    // TODO ask in the slack
    let mut parser = Parser::new(V8_SUPPORTED_FEATURES, StringInput::from(&*fm), None);

    // TODO
    // ok so these errors are recoverable, like we can successfully parse it.
    // if we wanted to be stricter, we could just Err here
    // we could also warn the user about these, but i think
    // we should first do some testing and figure out what level
    // of verbosity is appropriate.
    // my guess is that the V8 parser is better at recovering than swc
    // but i also know nothing about that. i just know google is a multi-billion
    // dollar company and swc is one guy
    let _ = parser.take_errors();

    // i have a feeling that a certain error-handling library would
    // make this match statement redundant...anyhow
    match parser.parse_script() {
        Ok(script) => Ok(script),
        Err(e) => Err(failure::format_err!("{:#?}", e)),
    }
}

fn check_ast_for_forbidden_knowledge(_ast: Vec<Stmt>) -> Result<(), failure::Error> {
    // i just keep writing code that is *not* the hard part
    todo!()
}
