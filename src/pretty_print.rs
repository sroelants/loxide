use crate::ast::Expr;

pub trait PrettyPrint {
    fn pretty_print(&self) -> String;
}

impl PrettyPrint for Expr {
    fn pretty_print(&self) -> String {
        match self {
            Expr::Grouping { expr } => format!("(group {expr})", expr = expr.pretty_print()),

            Expr::Binary { op, left, right } => format!(
                "({op} {left} {right})",
                op = op.lexeme,
                left = left.pretty_print(),
                right = right.pretty_print()
            ),

            Expr::Logical { op, left, right } => format!(
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

            Expr::Variable { name  } => format!("{name}"),

            Expr::Assignment { name, value  } => format!(
                "(assign {name} {value})",
                value = value.pretty_print()
            ),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{ast::LoxLiteral, span::Span, tokens::{Token, TokenType}};

    #[test]
    fn pretty_print() {
        let ast = Expr::Binary {
            op: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_owned(),
                span: Span::default()
            },
            left: Box::new(Expr::Unary {
                op: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_owned(),
                span: Span::default()
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
