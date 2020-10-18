use super::{AstNodeLinterArgs, Lintable};
use failure::format_err;
use swc_ecma_ast::{Decl, ModuleDecl, ModuleItem, Pat, VarDecl, VarDeclOrPat};

// other AstNodes that aren't expressions or statements

/// By implementing Lintable for Vec, we can call `ast.lint(args)`
/// at the top level and recurse through the whole AST. Plus it removes
/// a lot of other boilerplate code we'd otherwise have to write for other
/// expressions and statements that contain Vecs of lintable stuff.
///
/// Note: Ideally, the type signature would actually be more general, like
/// `impl<'a, T> Lintable<AstNodeLinterArgs<'a>> for T where T: Iterator<Item = dyn Lintable<AstNodeLinterArgs<'a>>>`,
/// but rustc is not happy about us implementing this when swc might potentially
/// implement Iterator for e.g. Stmt. Then we'd have conflicting implementations
/// of Lintable for any struct that also implemented Iterator.
/// For practical purposes though, this isn't a problem, as swc just uses Vec
/// for all groups of AstNodes
impl<'a, T> Lintable<AstNodeLinterArgs<'a>> for Vec<T>
where
    T: Lintable<AstNodeLinterArgs<'a>>,
{
    fn lint(&self, args: AstNodeLinterArgs<'a>) -> Result<(), failure::Error> {
        self.iter().try_for_each(|t| t.lint(args))
    }
}

/// This is similar to the implementation for Vec<T>, -- this
/// is mostly to prevent writing boilerplate.
impl<'a, T> Lintable<AstNodeLinterArgs<'a>> for Option<T>
where
    T: Lintable<AstNodeLinterArgs<'a>>,
{
    fn lint(&self, args: AstNodeLinterArgs<'a>) -> Result<(), failure::Error> {
        match self {
            Some(t) => t.lint(args),
            None => Ok(()),
        }
    }
}

// this stuff is just other nodes that need to be linted that aren't expressions or statements

impl<'a> Lintable<AstNodeLinterArgs<'a>> for ModuleItem {
    fn lint(&self, args: AstNodeLinterArgs<'a>) -> Result<(), failure::Error> {
        match self {
            ModuleItem::ModuleDecl(declaration) => declaration.lint(args),
            ModuleItem::Stmt(statement) => statement.lint(args),
        }
    }
}

impl<'a> Lintable<AstNodeLinterArgs<'a>> for ModuleDecl {
    fn lint(&self, args: AstNodeLinterArgs<'a>) -> Result<(), failure::Error> {
        match self {
            ModuleDecl::Import(import) => {
                if let Some(assertions) = import.asserts {
                    assertions.lint(args)?;
                };
                import.specifiers.lint(args)
            }
            ModuleDecl::ExportDecl(export) => export.decl.lint(args),
            ModuleDecl::ExportNamed(export) => export.specifiers.lint(args),
            ModuleDecl::ExportDefaultDecl(export) => match export.decl {
                swc_ecma_ast::DefaultDecl::Class(class) => class.lint(args),
                swc_ecma_ast::DefaultDecl::Fn(function) => function.lint(args),
                swc_ecma_ast::DefaultDecl::TsInterfaceDecl(interface) => {
                    format_err!("No typescript allowed!")
                }
            },
            ModuleDecl::ExportDefaultExpr(_) => {}
            ModuleDecl::ExportAll(_) => {}
            ModuleDecl::TsImportEquals(_) => {}
            ModuleDecl::TsExportAssignment(_) => {}
            ModuleDecl::TsNamespaceExport(_) => {}
        }
    }
}

impl<'a> Lintable<AstNodeLinterArgs<'a>> for Decl {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        todo!()
    }
}

impl<'a> Lintable<AstNodeLinterArgs<'a>> for Pat {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        todo!()
    }
}

impl<'a> Lintable<AstNodeLinterArgs<'a>> for VarDecl {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        todo!()
    }
}

impl<'a> Lintable<AstNodeLinterArgs<'a>> for VarDeclOrPat {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        match self {
            VarDeclOrPat::VarDecl(declaration) => declaration.lint(args),
            VarDeclOrPat::Pat(pattern) => pattern.lint(args),
        }
    }
}
