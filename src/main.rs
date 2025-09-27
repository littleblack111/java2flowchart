use java2flowchart::parser::parse;

fn main() {
    let test = "int a";

    println!("{:#?}", parse(test));
}
