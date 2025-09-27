use crate::parser::{Expr, ExprT, Metadata, Scope};

#[derive(Clone, Debug)]
pub enum DepthExpr<'a> {
    Decision {
        cond: Box<DepthExpr<'a>>,
        t: Scope,
        else_branch: Option<Box<Vec<DepthExpr<'a>>>>,
    },
    IO(&'a str),
    Process(&'a str),
}

pub fn parse<'a, 'b>(vec: &'b Vec<Expr<'a>>) -> Vec<DepthExpr<'a>> {
    let mut exprs: Vec<DepthExpr<'a>> = Vec::new();

    let mut it = vec
        .iter()
        .peekable();

    while let Some(e) = it.next() {
        if let ExprT::Decision((cond, scope)) = e.expr {
            let cond = DepthExpr::Process(cond);
            for e in it.clone() {
                if e.meta == Some(Metadata::EndScope) {
                    break;
                }
            }

            exprs.push(DepthExpr::Decision {
                cond: Box::new(cond),
                t: scope,
                else_branch: Some(Box::new(parse(&entire_scopes(
                    it.clone()
                        .cloned(),
                )))),
            });
        }
    }

    exprs
}

fn entire_scopes<'a>(it: impl Iterator<Item = Expr<'a>>) -> Vec<Expr<'a>> {
    let mut exprs: Vec<Expr<'a>> = Vec::new();
    let mut scope_count: u32 = 1; // assume we already entered the scope

    for i in it {
        if scope_count == 0 {
            break;
        }
        if let Some(meta) = i.meta {
            match meta {
                Metadata::StartScope => {
                    scope_count = scope_count
                        .checked_add(1)
                        .expect("Scopes nested too deep")
                }
                Metadata::EndScope => {
                    scope_count = scope_count
                        .checked_sub(1)
                        .expect("Closed not opened scope")
                }
            }
        }
        exprs.push(i);
    }

    exprs
}
