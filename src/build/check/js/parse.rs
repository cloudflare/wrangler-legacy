use std::{fs::File, path::PathBuf};

use sourcemap::SourceMap;
use swc_common::{sync::Lrc, SourceMap as SwcSourceMap};
use swc_ecma_parser::{Parser, StringInput};

use super::{JavaScript, Parseable, V8_SUPPORTED_JS_FEATURES};

impl Parseable<PathBuf> for JavaScript {
    fn parse(js_file: &PathBuf) -> Result<Self, failure::Error> {
        // it's worth noting that swc's SourceMap struct has nothing to do
        // with javascript source maps. it refers to a map of like, the in-memory
        // representation of the javascript as held on to by SWC
        let cm: Lrc<SwcSourceMap> = Default::default();
        let fm = cm.load_file(js_file)?;

        // should we actually do something about the comments?
        // TODO ask in the slack
        let mut parser = Parser::new(V8_SUPPORTED_JS_FEATURES, StringInput::from(&*fm), None);

        // these errors are recoverable
        let _ = parser.take_errors();

        let js_extension = js_file.extension().unwrap().to_str().unwrap();
        let map = js_file.with_extension(format!("{}.map", js_extension));

        let js_source_map = if map.is_file() {
            Some(SourceMap::from_reader(File::open(map)?)?)
        } else {
            // inline source map support when??
            None
        };

        match parser.parse_module() {
            Ok(module) => Ok(JavaScript {
                module,
                swc_source_map: cm,
                js_source_map,
            }),
            // if only there was a better error handling library...anyhow...
            Err(e) => Err(failure::format_err!("{:?}", e)),
        }
    }
}
