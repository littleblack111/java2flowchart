use std::iter::Peekable;

use crate::parser::{Expr, ExprT, Metadata, Scope};

#[derive(Clone, Debug)]
pub enum DepthExpr<'a> {
    Decision {
        cond: Box<DepthExpr<'a>>,
        t: Scope,
        then_branch: Box<Vec<DepthExpr<'a>>>,
        else_branch: Option<Box<Vec<DepthExpr<'a>>>>,
    },
    IO(&'a str),
    Process(&'a str),
}

pub fn parse<'a>(vec: &Vec<Expr<'a>>) -> Vec<DepthExpr<'a>> {
    let mut exprs: Vec<DepthExpr<'a>> = Vec::new();

    let mut it = vec
        .iter()
        .peekable();

    while let Some(e) = it.next() {
        match e.expr {
            ExprT::Decision((cond, scope)) => {
                let cond = DepthExpr::Process(cond);

                let then_branch = parse(&entire_scopes(&mut it));

                let else_branch = if let Some(n) = it.peek() {
                    match n.expr {
                        ExprT::Decision((s, _)) => {
                            if s.trim_start()
                                .starts_with("else")
                            {
                                it.next();
                                Some(Box::new(parse(&entire_scopes(&mut it))))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                };

                exprs.push(DepthExpr::Decision {
                    cond: Box::new(cond),
                    t: scope,
                    then_branch: Box::new(then_branch),
                    else_branch,
                });
            }
            ExprT::IO(s) => {
                if !s.is_empty() {
                    exprs.push(DepthExpr::IO(s))
                }
            }
            ExprT::Process(s) => {
                if !s.is_empty() {
                    exprs.push(DepthExpr::Process(s))
                }
            }
            ExprT::StartOrStop(_) => {}
        }
    }

    exprs
}

fn entire_scopes<'a>(it: &mut Peekable<std::slice::Iter<'_, Expr<'a>>>) -> Vec<Expr<'a>> {
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
        exprs.push(*i);
    }

    exprs
}
