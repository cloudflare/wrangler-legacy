use super::{AstNodeLinterArgs, Lintable};
use swc_ecma_ast::Expr;

/// [Expressions](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_Operators#Expressions)
/// are the things we actually care about linting. From MDN:
/// > An *expression* is any valid unit of code that resolves to a value
impl<'a> Lintable<AstNodeLinterArgs<'a>> for Expr {
    fn lint(&self, args: AstNodeLinterArgs) -> Result<(), failure::Error> {
        // I would like to reiterate, MDN is doing it like nobody else. Or, was doing it, I suppose.
        match self {
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this
            Expr::This(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array
            Expr::Array(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Object_initializer
            Expr::Object(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/function*
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/async_function
            Expr::Fn(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Unary_operators
            Expr::Unary(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Increment_and_decrement
            Expr::Update(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Binary_bitwise_operators
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Binary_logical_operators
            Expr::Bin(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators#Assignment_operators
            Expr::Assign(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Property_accessors
            Expr::Member(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Conditional_Operator
            Expr::Cond(expression) => expression.lint(args),
            // https://docs.onux.com/en-US/Developers/JavaScript-PP/Language/Reference/Expressions/function-call
            Expr::Call(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new
            Expr::New(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Comma_Operator
            Expr::Seq(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Glossary/Identifier
            Expr::Ident(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp
            // ...and other literals which don't need linting
            Expr::Lit(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals
            Expr::Tpl(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Template_literals#Tagged_templates
            Expr::TaggedTpl(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions
            Expr::Arrow(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/class
            Expr::Class(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/yield*
            Expr::Yield(expression) => expression.lint(args),
            // As far as I can tell, this is just... `new.target` ...
            // https://www.ecma-international.org/ecma-262/6.0/#sec-meta-properties
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/new.target
            Expr::MetaProp(_) => Ok(()),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await
            Expr::Await(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Grouping
            Expr::Paren(expression) => expression.lint(args),
            Expr::JSXMember(_) => Err(failure::err_msg("JSX is not allowed in workers!")),
            Expr::JSXNamespacedName(_) => Err(failure::err_msg("JSX is not allowed in workers!")),
            Expr::JSXEmpty(_) => Err(failure::err_msg("JSX is not allowed in workers!")),
            Expr::JSXElement(_) => Err(failure::err_msg("JSX is not allowed in workers!")),
            Expr::JSXFragment(_) => Err(failure::err_msg("JSX is not allowed in workers!")),
            Expr::TsTypeAssertion(_) => {
                Err(failure::err_msg("Typescript is not allowed in workers!"))
            }
            Expr::TsConstAssertion(_) => {
                Err(failure::err_msg("Typescript is not allowed in workers!"))
            }
            Expr::TsNonNull(_) => Err(failure::err_msg("Typescript is not allowed in workers!")),
            Expr::TsTypeCast(_) => Err(failure::err_msg("Typescript is not allowed in workers!")),
            Expr::TsAs(_) => Err(failure::err_msg("Typescript is not allowed in workers!")),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/Private_class_fields
            Expr::PrivateName(expression) => expression.lint(args),
            // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Optional_chaining
            Expr::OptChain(expression) => expression.lint(args),
            // TODO: we need to define a custom error type that's usable by match_to_source_map or
            // whatever it's called and throw that here instead of just failure::Error
            Expr::Invalid(_) => Err(failure::err_msg("Failed to parse expression!")),
        }
    }
}
