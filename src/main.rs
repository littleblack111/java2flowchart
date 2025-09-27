use java2flowchart::parser::{delimit, parse};

fn main() {
    let test = "if (a) {} else {}";

    println!("{:#?}", parse(delimit(test)));
}
