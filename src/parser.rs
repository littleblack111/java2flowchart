use crate::tokenizer::{self, If, Keyword::*, Loop};

pub type SplitWithMetadata<'a> = Vec<(&'a str, Option<Metadata>)>;

pub const EXPRESSION_DELIMITERS: &[char] = &[
    ';', '{', '}',
];

#[derive(Clone, Copy, Debug)]
pub enum Scope {
    If(If),
    Loop(Loop),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Metadata {
    StartScope,
    EndScope,
}

#[derive(Debug, Clone, Copy)]
pub enum ExprT<'a> {
    StartOrStop(bool),
    Decision((&'a str, Scope)),
    IO(&'a str),
    Process(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub struct Expr<'a> {
    pub expr: ExprT<'a>,
    pub meta: Option<Metadata>,
}

pub fn delimit(s: &str) -> SplitWithMetadata<'_> {
    s.split_inclusive([
        ';', '{', '}',
    ])
    .fold(Vec::new(), |mut acc, part| {
        if let Some((i, ch)) = part
            .char_indices()
            .next_back()
        {
            acc.push((
                if ch == ';' || ch == '{' || ch == '}' {
                    part[..i].trim()
                } else {
                    part.trim()
                },
                None,
            ));
            match ch {
                '{' => {
                    if let Some((_, meta)) = acc.last_mut() {
                        *meta = Some(Metadata::StartScope);
                    }
                }
                '}' => {
                    if let Some((_, meta)) = acc.last_mut() {
                        *meta = Some(Metadata::EndScope);
                    }
                }
                _ => {}
            }
        }
        acc
    })
}

pub fn process(strs: SplitWithMetadata<'_>) -> Vec<Expr<'_>> {
    let mut exprs = Vec::new();

    for (str, meta) in strs {
        let mut exp: Option<ExprT> = None;

        // TODO: fix one line if else as not delimited by {}
        for s in str.split_whitespace() {
            if let Some(token) = tokenizer::parse(if let len = s.len() {
                if s.starts_with("println") {
                    "println"
                } else if s.starts_with("throw") {
                    "throw"
                } else {
                    s
                }
            } else {
                s
            }) {
                match token {
                    If(t) => {
                        if let Some(ExprT::Decision((_, Scope::If(If::Else)))) = exp {
                        } else {
                            exp = Some(ExprT::Decision((str, Scope::If(t.to_owned()))));
                        }
                    }
                    Loop(t) => exp = Some(ExprT::Decision((str, Scope::Loop(t.to_owned())))),
                    Throw => exp = Some(ExprT::IO(str)),
                    IO => exp = Some(ExprT::IO(str)),
                }
            }
        }

        match exp {
            Some(expr) => exprs.push(Expr {
                expr,
                meta,
            }),
            None => exprs.push(Expr {
                expr: ExprT::Process(str),
                meta,
            }),
        }
    }

    exprs
}

pub fn parse(str: &str) -> Vec<Expr<'_>> {
    process(delimit(str))
}
