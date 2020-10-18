use sourcemap::SourceMap;
use swc_ecma_ast::{Module, Script};

use super::{ExpressionList, Lintable};

mod expressions;
mod misc;
mod statements;

// the difference between the args for linting a Script and linting an AstNode
// is that the script doesn't need to know whether or not it's in the request context,
// because it's always *not* in the request context. It does, however, take an optional
// source map that can be used to map errors to the original source to provide more
// helpful error messages to developers
type ScriptLinterArgs<'a> = (Option<&'a SourceMap>, ExpressionList, ExpressionList);
type AstNodeLinterArgs<'a> = (bool, &'a ExpressionList, &'a ExpressionList);

impl<'a> Lintable<ScriptLinterArgs<'a>> for Script {
    fn lint(
        &self,
        (source_map, unavailable, available_in_request_context): ScriptLinterArgs,
    ) -> Result<(), failure::Error> {
        if let Err(error) = self
            .body
            .lint((false, &unavailable, &available_in_request_context))
        {
            Err(match source_map {
                Some(map) => match_error_to_source_map(error, map)?,
                None => error,
            })
        } else {
            Ok(())
        }
    }
}

impl<'a> Lintable<ScriptLinterArgs<'a>> for Module {
    fn lint(
        &self,
        (source_map, unavailable, available_in_request_context): ScriptLinterArgs<'a>,
    ) -> Result<(), failure::Error> {
        if let Err(error) = self
            .body
            .lint((false, &unavailable, &available_in_request_context))
        {
            Err(match source_map {
                Some(map) => match_error_to_source_map(error, map)?,
                None => error,
            })
        } else {
            Ok(())
        }
    }
}

// TODO: it would be cool to have line numbers in the errors
// and i don't think it would be like extremely hard to do,
// since every statement has its own associated byte position.
// But that's a nice-to-have for sure
fn match_error_to_source_map(
    error: failure::Error,
    source_map: &SourceMap,
) -> Result<failure::Error, failure::Error> {
    Ok(failure::format_err!("Thanks for providing us with a source map! Soon hopefully we will be able to tell you what part of your original source code is bad. Unfortunately, for now, all we can say is\n{}", error))
}
