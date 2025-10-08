use std::path::Path;

use java2flowchart::ast::parse;
use java2flowchart::image::FlowChart;
use java2flowchart::parser;

fn main() {
    let test = "first; if (first) {firstthen; if (firstthenif) {firstthenifthen} else {firstthenelse}} else {firstelse}println('a'); second; thirdlongprocess";

    let ast = parse(&parser::parse(test));

    println!("{:#?}", ast);

    FlowChart::create(&ast, Path::new("output.png"));
}
