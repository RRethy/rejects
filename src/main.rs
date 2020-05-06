mod parser;

fn main() {
    let res = parser::Parser::parse("(a|b)*(http:|https:)//www google com+");
    if let Ok(regex) = res {
        println!("{:?}", regex);
        println!("{}", regex.find("http://www google com"));
        println!("{}", regex.find("abababaaahttp://www google com"));
        println!("{}", regex.find("https://www google commmmm"));
        println!("{}", regex.find("https://www google com"));
        println!("{}", regex.find("http://www google com"));
        println!("{}", regex.find("http://www google co"));
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
