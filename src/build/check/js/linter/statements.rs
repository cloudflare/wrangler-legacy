use swc_ecma_ast::{
    BlockStmt, DoWhileStmt, ExprStmt, ForInStmt, ForOfStmt, ForStmt, IfStmt, LabeledStmt,
    ReturnStmt, Stmt, SwitchStmt, ThrowStmt, TryStmt, VarDeclOrExpr, WhileStmt, WithStmt,
};

use super::{AstNodeLinterArgs, Lintable};

impl<'a> Lintable<AstNodeLinterArgs<'a>> for Stmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        // tremendous shoutout to MDN, shame they shut it down
        match self {
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block
            Stmt::Block(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/Empty
            Stmt::Empty(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/debugger
            Stmt::Debugger(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with
            Stmt::With(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return
            Stmt::Return(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/label
            Stmt::Labeled(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break
            Stmt::Break(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue
            Stmt::Continue(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else
            Stmt::If(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch
            Stmt::Switch(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw
            Stmt::Throw(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch
            Stmt::Try(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while
            Stmt::While(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while
            Stmt::DoWhile(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for
            Stmt::For(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...in
            Stmt::ForIn(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of
            Stmt::ForOf(statement) => statement.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements#Declarations
            Stmt::Decl(statement) => statement.lint(args),
            // i suppose all expressions are technically statements?
            Stmt::Expr(statement) => statement.lint(args),
        }
    }
}

/// [Block statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/block)
/// are just any block of code in between some
/// curly braces, so we can treat them like a mini-AST and just
/// lint all of their child statements.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for BlockStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.stmts.lint(args)
    }
}

/// [With statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/with)
/// are...deprecated? I personally have never seen them used, but it acts just like a with
/// statement in Python -- it exposes whatever is in the with expression to its child scope
/// ```ignore
/// var a, x, y;
/// var r = 10;
///
/// with (Math) {
///   a = PI * r * r;
///   x = r * cos(PI);
///   y = r * sin(PI / 2);
/// }
/// ```
impl<'a> Lintable<AstNodeLinterArgs<'a>> for WithStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.obj.lint(args)?;
        self.body.lint(args)?;
        Ok(())
    }
}

/// [Return statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return)
/// can either return an expression or nothing. If they return an expression, we need to lint it.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for ReturnStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        if let Some(expression) = &self.arg {
            expression.lint(args)
        } else {
            Ok(())
        }
    }
}

/// [Labeled statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/label)
/// allow for break of continue statements to refer to their target with a label
impl<'a> Lintable<AstNodeLinterArgs<'a>> for LabeledStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.body.lint(args)
    }
}

/// [If statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/if...else)
/// contain a test expression, which needs to be linted, and a consequent body that gets executed if the statement is
/// true -- which also needs to be linted. Optionally, they may contain an `else` clause, which also also
/// needs to be linted.
///
/// Not entirely sure how this handled multiple `if else` statements, but I'm sure it's fine.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for IfStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.test.lint(args)?;
        self.cons.lint(args)?;

        if let Some(else_statement) = &self.alt {
            else_statement.lint(args)
        } else {
            Ok(())
        }
    }
}

/// [Switch statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/switch)
/// contain a discriminant expression, which needs to be linted, and a bunch of cases, which also need
/// to be linted. Every one of these cases except for `default` contains a test expression, which needs to be linted.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for SwitchStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.discriminant.lint(args)?;
        self.cases.iter().try_for_each(|case| {
            if let Some(expression) = &case.test {
                expression.lint(args)?;
            };
            case.cons.lint(args)
        })
    }
}

/// [Throw statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/throw)
/// have an expression that they throw, which needs to be linted.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for ThrowStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.arg.lint(args)
    }
}

/// [Try statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/try...catch)
/// contain a block of code inside the `try {}`, which needs to be linted. Optionally, there may also be
/// a `catch {}` clause, which needs to be linted. If the `catch` is catching something specific, that
/// expression also needs to be linted. Finally, if there's a `finally`, the content of that statement
/// needs to be linted, too.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for TryStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        // lint the stuff inside try {}
        self.block.lint(args)?;

        // lint the stuff inside catch {}, if there is one
        if let Some(clause) = &self.handler {
            clause.body.lint(args)?;

            // lint the specifically caught error, if it exists
            // TODO: do we actually need to do this?
            if let Some(pattern) = &clause.param {
                pattern.lint(args)?;
            }
        };

        // lint the finally {}, if it exists
        if let Some(finally) = &self.finalizer {
            finally.lint(args)
        } else {
            Ok(())
        }
    }
}

/// [While statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while)
/// test to see if a condition is true, and executes a block if it is. Both the test and the block
/// need to be linted.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for WhileStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.test.lint(args)?;
        self.body.lint(args)
    }
}

///[Do-While statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/do...while)
/// function the same as `while` statements, except that the test comes after the block, guaranteeing the
/// block is run at least once, even if the condition evaluates to false.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for DoWhileStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.test.lint(args)?;
        self.body.lint(args)
    }
}

/// [For statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for)
/// contain several elements that need linting. Consider the following:
/// ```ignore
/// for(let i = 0; i < arr.len; i++) {
///     // do stuff
/// }
/// ```
/// * the entire `for ... {}` block refers to the ForStmt
/// * the `let i = 0` expression, or initializer, needs to be linted
/// * the `i < arr.len` expression, or test, needs to be linted
/// * the i++ expression, or update, needs to be linted
/// * the contents of the block need to be linted
///
/// Due to the loose nature of javascript, many of these elements are optional, hence
/// the usage of `match` and `if let Some` statements.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for ForStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        match &self.init {
            Some(VarDeclOrExpr::VarDecl(declaration)) => declaration.lint(args),
            Some(VarDeclOrExpr::Expr(expression)) => expression.lint(args),
            None => Ok(()),
        }?;

        if let Some(expression) = &self.test {
            expression.lint(args)?
        };

        if let Some(expression) = &self.update {
            expression.lint(args)?
        };

        self.body.lint(args)
    }
}

/// [For...in statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...in)
/// have three parts that need linting -- the "left" expression, "right" expression, and the body of the loop.
/// It's easier to explain with an example.
///
/// ```ignore
/// for (const e in arr) {
///     // do stuff
/// }
///
/// * `e` is the left expression
/// * `arr` is the right expression
/// * `// do stuff` is the body of the loop
impl<'a> Lintable<AstNodeLinterArgs<'a>> for ForInStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.left.lint(args)?;
        self.right.lint(args)?;
        self.body.lint(args)
    }
}

/// [For...of statements](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...of)
/// function similarly to `for...in` statements, except for objects instead of arrays.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for ForOfStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.left.lint(args)?;
        self.right.lint(args)?;
        self.body.lint(args)
    }
}

/// As far as I can tell, the ExprStmt struct is for statements that are just expressions,
/// like if you made a `fetch()` call without assigning the result to anything. These are
/// easy to lint because you just have to lint the expression.
impl<'a> Lintable<AstNodeLinterArgs<'a>> for ExprStmt {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        self.expr.lint(args)
    }
}
