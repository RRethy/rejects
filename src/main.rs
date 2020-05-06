mod parser;

fn main() {
    let res = parser::Parser::parse("(abc|abd|a)*");
    if let Ok(regex) = res {
        println!("{:?}", regex);
        println!("{}", regex.find("ab"));
        println!("{}", regex.find("dabc"));
        println!("{}", regex.find("abc"));
        println!("{}", regex.find("abd"));
        println!("{}", regex.find("abdabcaaaaa"));
        println!("{}", regex.find("aaaa"));
        println!("{}", regex.find("a"));
        println!("{}", regex.find(""));
    } else {
        println!("Error");
    }
}
