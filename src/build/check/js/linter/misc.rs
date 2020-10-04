use super::{AstNodeLinterArgs, Lintable};
use swc_ecma_ast::{Decl, Pat, VarDecl, VarDeclOrPat};

// other AstNodes that aren't expressions or statements

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
