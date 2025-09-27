use crate::tokenizer::{self, If, Keyword::*, Loop};

pub const EXPRESSION_DELIMITERS: &[char] = &[
    ';', '{', '}',
];

#[derive(Clone, Copy, Debug)]
pub enum Scope {
    If(If),
    Loop(Loop),
}

#[derive(Debug)]
pub enum Expr<'a> {
    StartOrStop(bool),
    Decision((&'a str, Scope)),
    IO(&'a str),
    Process(&'a str),
}

pub fn delimit(str: &str) -> Vec<&str> {
    str.split(|c| EXPRESSION_DELIMITERS.contains(&c))
        .filter(|c| !c.is_empty())
        .collect()
}

pub fn process(strs: Vec<&str>) -> Vec<Expr<'_>> {
    let mut exprs = Vec::new();

    for str in strs {
        let mut exp: Option<Expr> = None;

        // TODO: fix one line if else as not delimited by {}
        for s in str.split_whitespace() {
            if let Some(token) = tokenizer::parse(s) {
                exp = match token {
                    If(t) => Some(Expr::Decision((str, Scope::If(t.to_owned())))),
                    Loop(t) => Some(Expr::Decision((str, Scope::Loop(t.to_owned())))),
                    Throw => Some(Expr::IO(str)),
                    IO => Some(Expr::IO(str)),
                }
            }
        }

        match exp {
            Some(exp) => exprs.push(exp),
            None => exprs.push(Expr::Process(str)),
        }
    }

    exprs
}

pub fn parse(str: &str) -> Vec<Expr<'_>> {
    process(delimit(str))
}
