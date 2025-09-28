use java2flowchart::ast::parse;
use java2flowchart::parser;

fn main() {
    let test = "if (first) {firstthen; if (firstthenif) {firstthenifthen} else {firstthenelse}} else {firstelse}println('a')";

    println!("{:#?}", parse(&parser::parse(test)));
}
