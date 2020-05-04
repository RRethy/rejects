mod parser;

fn main() {
    println!("Hello, world!");
    let output = parser::Parser::parse("a*");
    if let Some(f) = output {
        println!("{}", f.matches("aaaaaaaaaa"));
        println!("{}", f.matches("aaaabaaaaa"));
    } else {
        println!("Error");
    }
}
