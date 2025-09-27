use phf::{Map, phf_map};

#[derive(Clone, Copy, Debug)]
pub enum If {
    If,
    Else,
}

#[derive(Clone, Copy, Debug)]
pub enum Loop {
    For,
    While,
}

pub enum Keyword {
    If(If),
    Loop(Loop),
    Throw,
    IO,
}

pub static KEYWORDS: Map<&'static str, Keyword> = phf_map! {
    "if" => Keyword::If(If::If),
    "else" => Keyword::If(If::Else),
    "for" => Keyword::Loop(Loop::For),
    "while" => Keyword::Loop(Loop::While),
    "throw" => Keyword::Throw,
    "println" => Keyword::IO,
    "print" => Keyword::IO,
};

pub fn parse(keyword: &str) -> Option<&Keyword> {
    KEYWORDS.get(keyword)
}
