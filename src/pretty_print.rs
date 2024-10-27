use crate::ast::Expr;

pub trait PrettyPrint {
    fn pretty_print(&self) -> String;
}

impl<'a> PrettyPrint for Expr<'a> {
    fn pretty_print(&self) -> String {
        match self {
            Expr::Grouping { expr } => format!("(group {expr})", expr = expr.pretty_print()),

            Expr::Binary { op, left, right } => format!(
                "({op} {left} {right})",
                op = op.lexeme,
                left = left.pretty_print(),
                right = right.pretty_print()
            ),

            Expr::Unary { op, right } => format!(
                "({op} {right})",
                op = op.lexeme,
                right = right.pretty_print()
            ),

            Expr::Literal { value } => format!("{value}"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{ast::LoxLiteral, scanner::{Token, TokenType}};

    #[test]
    fn pretty_print() {
        let ast = Expr::Binary {
            op: Token {
                token_type: TokenType::Star,
                lexeme: "*",
                line: 1,
                col: 1,
            },
            left: Box::new(Expr::Unary {
                op: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-",
                    line: 1,
                    col: 1,
                },
                right: Box::new(Expr::Literal {
                    value: LoxLiteral::Num(123.0),
                }),
            }),
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    value: LoxLiteral::Num(45.67),
                }),
            }),
        };

        assert_eq!(ast.pretty_print(), "(* (- 123) (group 45.67))")
    }
}
