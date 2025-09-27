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
pub enum Expression<'a> {
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

pub fn parse(strs: Vec<&str>) -> Vec<Expression<'_>> {
    let mut expressions = Vec::new();

    for str in strs {
        let mut exp: Option<Expression> = None;

        // TODO: fix one line if else as not delimited by {}
        for s in str.split_whitespace() {
            if let Some(token) = tokenizer::parse(s) {
                exp = match token {
                    If(t) => Some(Expression::Decision((str, Scope::If(t.to_owned())))),
                    Loop(t) => Some(Expression::Decision((str, Scope::Loop(t.to_owned())))),
                    Throw => Some(Expression::IO(str)),
                    IO => Some(Expression::IO(str)),
                }
            }
        }

        match exp {
            Some(exp) => expressions.push(exp),
            None => expressions.push(Expression::Process(str)),
        }
    }

    expressions
}
