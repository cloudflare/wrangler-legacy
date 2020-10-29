use sourcemap::SourceMap;
use swc_common::{sync::Lrc, FileLines, SourceMap as SwcSourceMap, Span, Spanned};
use swc_ecma_ast::*;
use swc_ecma_visit::{Node, Visit, VisitWith};

use super::{JavaScript, Lintable};

impl Lintable for JavaScript {
    fn lint(&self) -> Result<(), failure::Error> {
        let mut linter = JavaScriptLinter::new(&self.swc_source_map);
        self.module.visit_children_with(&mut linter);
        if linter.errors.is_empty() {
            Ok(())
        } else {
            Err(linter.aggregate_errors(self.js_source_map.as_ref()))
        }
    }
}

struct JavaScriptLinter {
    source_map: Lrc<SwcSourceMap>,
    pub errors: Vec<JavaScriptLinterError>,
}

impl JavaScriptLinter {
    pub fn new(source_map: &Lrc<SwcSourceMap>) -> Self {
        JavaScriptLinter {
            source_map: source_map.clone(),
            errors: vec![],
        }
    }

    pub fn error<S: Into<String>>(&mut self, node: &dyn Spanned, reason: S) {
        let error = JavaScriptLinterError::new(node.span(), reason.into(), &self.source_map);
        self.errors.push(error);
    }

    pub fn aggregate_errors(&self, source_map: Option<&SourceMap>) -> failure::Error {
        let messages = if let Some(_map) = source_map {
            self.errors
                .iter()
                .map(|_error| {
                    // map the errors to their original locations in the source files
                    todo!()
                })
                .collect::<Vec<String>>()
        } else {
            self.errors
                .iter()
                .map(|error| format!("{}", error))
                .collect()
        };

        failure::err_msg(messages.join("\n"))
    }
}

impl Visit for JavaScriptLinter {
    fn visit_module_decl(&mut self, n: &ModuleDecl, _parent: &dyn Node) {
        match n {
            ModuleDecl::Import(import) => {
                if import.type_only {
                    self.error(import, "Typescript not allowed!")
                }
            }
            ModuleDecl::ExportDecl(_) => {}
            ModuleDecl::ExportNamed(_) => {}
            ModuleDecl::ExportDefaultDecl(_) => {}
            ModuleDecl::ExportDefaultExpr(_) => {}
            ModuleDecl::ExportAll(_) => {}
            ModuleDecl::TsImportEquals(ts) => self.error(ts, "Typescript not allowed!"),
            ModuleDecl::TsExportAssignment(ts) => self.error(ts, "Typescript not allowed!"),
            ModuleDecl::TsNamespaceExport(ts) => self.error(ts, "Typescript not allowed!"),
        }
    }
}

// TODO this should take into account a source map provided by the user
// and try to map errors to the source if possible
struct JavaScriptLinterError {
    location: FileLines,
    reason: String,
}

impl JavaScriptLinterError {
    fn new(span: Span, reason: String, source_map: &Lrc<SwcSourceMap>) -> JavaScriptLinterError {
        JavaScriptLinterError {
            location: source_map.span_to_lines(span).unwrap(),
            reason,
        }
    }
}

impl std::fmt::Debug for JavaScriptLinterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JavaScriptLinterError")
            .field("reason", &self.reason)
            .field("file", &self.location.file.name)
            .field("lines", &self.location.lines)
            .finish()
    }
}

impl std::fmt::Display for JavaScriptLinterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at {} {:?}: {}",
            self.location.file.name, self.location.lines, self.reason
        )
    }
}

impl failure::Fail for JavaScriptLinterError {}
