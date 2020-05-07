extern crate rejects;

fn main() {
    let res = rejects::compile(r"\w\w\w\d\d\d\W\W");
    // let res = parser::Parser::parse("(a|b)*(http:|https:)//www google com+");
    if let Ok(regex) = res {
        println!("{:?}", regex);
        println!("{}", regex.find("abc123=-"));
        println!("{}", regex.find("ABc123 	"));
        println!("{}", regex.find("Ac1234 	"));
        println!("{}", regex.find("1"));
        println!("{}", regex.find("2"));
        println!("{}", regex.find("3"));
        println!("{}", regex.find("a"));
        println!("{}", regex.find("b"));
        println!("{}", regex.find("c"));
        println!("{}", regex.find("d"));
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
    } else if let Err(e) = res {
        println!("Error: {:?}", e);
    }
}
