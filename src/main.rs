use rejects::rejects::Rejects;

fn main() {
    let res = Rejects::new(r"(\d\d\d\d)-(\d\d)-(\d\d)");
    // let res = parser::Parser::parse("(a|b)*(http:|https:)//www google com+");
    if let Ok(regex) = res {
        println!("{:?}", regex);
        println!("{}", regex.is_match("2010-03-14"));
        println!("{}", regex.is_match("ABc123 	"));
        println!("{}", regex.is_match("Ac1234 	"));
        println!("{}", regex.is_match("1"));
        println!("{}", regex.is_match("2"));
        println!("{}", regex.is_match("3"));
        println!("{}", regex.is_match("a"));
        println!("{}", regex.is_match("b"));
        println!("{}", regex.is_match("c"));
        println!("{}", regex.is_match("d"));
        println!("{}", regex.is_match("abababaaahttp://www google com"));
        println!("{}", regex.is_match("https://www google commmmm"));
        println!("{}", regex.is_match("https://www google com"));
        println!("{}", regex.is_match("http://www google com"));
        println!("{}", regex.is_match("http://www google co"));
        println!("{}", regex.is_match("dabc"));
        println!("{}", regex.is_match("abc"));
        println!("{}", regex.is_match("abd"));
        println!("{}", regex.is_match("abdabcaaaaa"));
        println!("{}", regex.is_match("aaaa"));
        println!("{}", regex.is_match("a"));
        println!("{}", regex.is_match(""));
    } else if let Err(e) = res {
        println!("Error: {:?}", e);
    }
}
