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

#[derive(Debug)]
pub enum Metadata {
    StartScope,
    EndScope,
}

#[derive(Debug)]
pub enum ExprT<'a> {
    StartOrStop(bool),
    Decision((&'a str, Scope)),
    IO(&'a str),
    Process(&'a str),
}

#[derive(Debug)]
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
            let body = if ch == ';' || ch == '{' || ch == '}' {
                part[..i].trim()
            } else {
                part.trim()
            };
            if !body.is_empty() {
                acc.push((body, None));
            }
            match ch {
                '{' => {
                    if let Some(last) = acc.last_mut() {
                        last.1 = Some(Metadata::StartScope);
                    }
                }
                '}' => {
                    if let Some(last) = acc.last_mut() {
                        last.1 = Some(Metadata::EndScope);
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
            if let Some(token) = tokenizer::parse(s) {
                exp = match token {
                    If(t) => Some(ExprT::Decision((str, Scope::If(t.to_owned())))),
                    Loop(t) => Some(ExprT::Decision((str, Scope::Loop(t.to_owned())))),
                    Throw => Some(ExprT::IO(str)),
                    IO => Some(ExprT::IO(str)),
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
                meta: None,
            }),
        }
    }

    exprs
}

pub fn parse(str: &str) -> Vec<Expr<'_>> {
    process(delimit(str))
}
