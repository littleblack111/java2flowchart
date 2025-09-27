use java2flowchart::ast::parse;
use java2flowchart::parser;

fn main() {
    let test = "if (a) {if (a) {a} else if {asd else if {}}}";

    println!("{:#?}", parse(&parser::parse(test)));
}
