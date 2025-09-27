use crate::parser::{self, Expr, ExprT, Scope};

pub enum DepthExpr<'a> {
    Decision {
        cond: Box<DepthExpr<'a>>,
        t: Scope,
        else_branch: Option<Box<DepthExpr<'a>>>,
    },
    IO(&'a str),
    Process(&'a str),
}

fn parse(vec: Vec<Expr>) -> Vec<DepthExpr> {
    let exprs = Vec::new();

    let mut it = vec
        .iter()
        .peekable();

    while let Some(e) = it.next() {
        if let ExprT::Decision((cond, scope)) = e.expr {
            let cond = parser::parse(cond);
        }
    }

    exprs
}
