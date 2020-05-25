fn main() {
    let re = rejects::make_rejects!(r"((\d\d\d\d)-(\d\d)-(\d\d))+");
    println!("{}", re.find_end("2010-03-14"));
    println!("{}", re.find_end("2010-03-142010-03-142010-03-142010-03-142010-03-142010-03-142010-03-14aaaa"));
    println!("{}", re.find_end("2010-03-142010-03-14"));
    println!("{}", re.find_end("2010-03-14 ss"));
    println!("{}", re.find_end("Ac1234 	"));
    println!("{}", re.find_end("1"));
    println!("{}", re.find_end("2"));
    println!("{}", re.find_end("3"));
    println!("{}", re.find_end("a"));
    println!("{}", re.find_end("b"));
    println!("{}", re.find_end("c"));
    println!("{}", re.find_end("d"));
    println!("{}", re.find_end("abababaaahttp://www google com"));
    println!("{}", re.find_end("https://www google commmmm"));
    println!("{}", re.find_end("https://www google com"));
    println!("{}", re.find_end("http://www google com"));
    println!("{}", re.find_end("http://www google co"));
    println!("{}", re.find_end("dabc"));
    println!("{}", re.find_end("abc"));
    println!("{}", re.find_end("abd"));
    println!("{}", re.find_end("abdabcaaaaa"));
    println!("{}", re.find_end("aaaa"));
    println!("{}", re.find_end("a"));
    println!("{}", re.find_end(""));
}
