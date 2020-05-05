mod parser;

fn main() {
    let res = parser::Parser::parse("abc");
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
