use std::path::PathBuf;

use sourcemap::SourceMap;
use swc_common::{sync::Lrc, SourceMap as SwcSourceMap};
use swc_ecma_ast::{Expr, Module};
use swc_ecma_parser::{Parser, StringInput};

use super::{Lintable, Parseable, Validate};

mod linter;

#[cfg(test)]
mod tests;

use super::config::{
    AVAILABLE_WITHIN_REQUEST_CONTEXT, UNAVAILABLE_BUILTINS, V8_SUPPORTED_JS_FEATURES,
};

pub struct JavaScript {
    module: Module,
    source_map: Option<SourceMap>,
}

// https://github.com/rust-lang/rust/issues/63063
// type ExpressionList = impl Iterator<Item = Expr>;
type ExpressionList = Vec<Expr>;
pub type JavaScriptLinterArgs = (ExpressionList, ExpressionList);

impl Lintable<JavaScriptLinterArgs> for JavaScript {
    fn lint(
        &self,
        (unavailable, available_in_request_context): JavaScriptLinterArgs,
    ) -> Result<(), failure::Error> {
        self.module.lint((
            self.source_map.as_ref(),
            unavailable,
            available_in_request_context,
        ))
    }
}

impl Validate<(ExpressionList, ExpressionList), (PathBuf, Option<PathBuf>)> for JavaScript {
    fn validate(&self) -> Result<(), failure::Error> {
        self.lint((
            UNAVAILABLE_BUILTINS.into(),
            AVAILABLE_WITHIN_REQUEST_CONTEXT.into(),
        ))
    }
}

impl Parseable<(PathBuf, Option<PathBuf>)> for JavaScript {
    fn parse(
        (js_file, _source_map_file): &(PathBuf, Option<PathBuf>),
    ) -> Result<Self, failure::Error> {
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
        let cm: Lrc<SwcSourceMap> = Default::default();

        let fm = cm.load_file(js_file)?;

        // should we actually do something about the comments?
        // TODO ask in the slack
        let mut parser = Parser::new(V8_SUPPORTED_JS_FEATURES, StringInput::from(&*fm), None);

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

        match parser.parse_module() {
            Ok(module) => Ok(JavaScript {
                module,
                // TODO: parse source map
                source_map: None,
            }),
            // if only there was a better error handling library...anyhow...
            Err(e) => Err(failure::format_err!("{:?}", e)),
        }
    }
}
