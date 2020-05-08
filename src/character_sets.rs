use std::collections::HashSet;

pub(crate) fn word_chars() -> HashSet<char> {
    let mut set = HashSet::new();
    for c in b'a'..=b'z' {
        set.insert(c as char);
    }
    for c in b'A'..=b'Z' {
        set.insert(c as char);
    }
    for c in b'0'..=b'9' {
        set.insert(c as char);
    }
    set.insert('_');
    set
}

pub(crate) fn digits() -> HashSet<char> {
    let mut set = HashSet::new();
    for c in b'0'..=b'9' {
        set.insert(c as char);
    }
    set
}

pub(crate) fn whitespace() -> HashSet<char> {
    let mut set = HashSet::new();
    set.insert(' ');
    set.insert('\t');
    set
}

pub(crate) fn range(low: u8, high: u8) -> Result<HashSet<char>, (u8, u8)> {
    if high < low {
        return Err((low, high));
    }
    let mut set = HashSet::new();
    for c in low..high {
        set.insert(c as char);
    }
    Ok(set)
}
