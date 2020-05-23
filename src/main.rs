use rejects::rejects::Rejects;

fn main() {
    let res = Rejects::new(r"(\d\d\d\d)-(\d\d)-(\d\d)");
    // let res = parser::Parser::parse("(a|b)*(http:|https:)//www google com+");
    if let Ok(regex) = res {
        // println!("{:?}", regex);
        println!("{}", regex.find_end("2010-03-14"));
        println!("{}", regex.find_end("2010-03-14 ss"));
        println!("{}", regex.find_end("Ac1234 	"));
        println!("{}", regex.find_end("1"));
        println!("{}", regex.find_end("2"));
        println!("{}", regex.find_end("3"));
        println!("{}", regex.find_end("a"));
        println!("{}", regex.find_end("b"));
        println!("{}", regex.find_end("c"));
        println!("{}", regex.find_end("d"));
        println!("{}", regex.find_end("abababaaahttp://www google com"));
        println!("{}", regex.find_end("https://www google commmmm"));
        println!("{}", regex.find_end("https://www google com"));
        println!("{}", regex.find_end("http://www google com"));
        println!("{}", regex.find_end("http://www google co"));
        println!("{}", regex.find_end("dabc"));
        println!("{}", regex.find_end("abc"));
        println!("{}", regex.find_end("abd"));
        println!("{}", regex.find_end("abdabcaaaaa"));
        println!("{}", regex.find_end("aaaa"));
        println!("{}", regex.find_end("a"));
        println!("{}", regex.find_end(""));
    } else if let Err(e) = res {
        println!("Error: {:?}", e);
    }
}
