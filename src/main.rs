use java2flowchart::parser::parse;

fn main() {
    let test = "if (a) {f}";

    println!("{:#?}", parse(test));
}
