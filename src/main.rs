mod parser;

fn main() {
    let res = parser::Parser::parse("(abc|dabc)");
    if let Ok(regex) = res {
        println!("{:?}", regex);
        println!("{}", regex.find("abc"));
        println!("{}", regex.find("abd"));
        println!("{}", regex.find("ab"));
        println!("{}", regex.find("dabc"));
    } else {
        println!("Error");
    }
}
