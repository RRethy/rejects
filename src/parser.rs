use crate::character_sets;
use crate::nfa::{Fragment, State, StateList};
use crate::rejects::Rejects;
use std::collections::HashSet;
use std::iter::Peekable;
use std::str::Chars;

#[allow(dead_code)]
pub struct Parser<'a> {
    iter: Peekable<Chars<'a>>,
    index: u32,
    errors: Vec<u32>,
}

pub fn parse(s: &str) -> Result<Rejects, Vec<u32>> {
    let mut parser = Parser::new(s);
    let mut statelist = StateList::new();
    if let Some(frag) = parser.parse_union(&mut statelist) {
        let match_state = statelist.add_state(State::make_match());
        for &dangler in frag.endstates.iter() {
            statelist.link(dangler, match_state);
        }
        // ensure we are at the end of the string
        if let Some(_) = parser.iter.next() {
            parser.error_next();
        }
        if parser.errors.len() > 0 {
            return Err(parser.errors);
        }
        return Ok(Rejects::new(frag.start, statelist.states));
    }
    // ensure we are at the end of the string
    if let Some(_) = parser.iter.next() {
        parser.error_next();
    }
    if parser.errors.len() > 0 {
        return Err(parser.errors);
    }
    Err(Vec::new())
}

impl<'a> Parser<'a> {
    fn new<'b: 'a>(s: &'b str) -> Parser<'a> {
        Parser {
            iter: s.chars().peekable(),
            index: 0,
            errors: Vec::new(),
        }
    }

    fn parse_union(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                let l = self.parse_concat(statelist);
                let r = self.parse_union_prime(statelist);
                statelist.union(l, r)
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => {
                self.error_next();
                None
            }
            Some(_) => {
                let l = self.parse_concat(statelist);
                let r = self.parse_union_prime(statelist);
                statelist.union(l, r)
            }
            None => {
                self.error_next();
                None
            }
        }
    }

    fn parse_union_prime(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some(')') => None,
            Some('|') => {
                self.consume();
                self.parse_union(statelist)
            }
            Some(_) => {
                self.error_next();
                None
            }
            None => None,
        }
    }

    fn parse_concat(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                let l = self.parse_unary(statelist);
                let r = self.parse_concat_prime(statelist);
                statelist.concatenation(l, r)
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => {
                self.error_next();
                None
            }
            Some(_) => {
                let l = self.parse_unary(statelist);
                let r = self.parse_concat_prime(statelist);
                statelist.concatenation(l, r)
            }
            None => {
                self.error_next();
                None
            }
        }
    }

    fn parse_concat_prime(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => self.parse_concat(statelist),
            Some(')') => None,
            Some('*') | Some('?') | Some('+') => {
                self.error_next();
                None
            }
            Some('|') => None,
            Some(_) => self.parse_concat(statelist),
            None => None,
        }
    }

    fn parse_unary(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                let l = self.parse_paren(statelist);
                let r = self.parse_unaryop();
                statelist.unary_operator(l, r)
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => {
                self.error_next();
                None
            }
            Some(_) => {
                let l = self.parse_paren(statelist);
                let r = self.parse_unaryop();
                statelist.unary_operator(l, r)
            }
            None => {
                self.error_next();
                None
            }
        }
    }

    fn parse_unaryop(&mut self) -> Option<char> {
        match self.iter.peek() {
            Some('(') => None,
            Some(')') => None,
            Some('?') | Some('*') | Some('+') => self.consume(),
            Some('|') => None,
            Some(_) => None,
            None => None,
        }
    }

    fn parse_paren(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                self.consume();
                let fragment = self.parse_union(statelist);
                if let Some(')') = self.iter.peek() {
                    self.consume();
                    fragment
                } else {
                    self.error_next();
                    None
                }
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') | None => {
                self.error_next();
                None
            }
            Some(_) => self.parse_term(statelist),
        }
    }

    fn parse_term(&mut self, statelist: &mut StateList) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') | Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => {
                self.error_next();
                None
            }
            Some('.') => Some(statelist.characters(HashSet::new())),
            Some('\\') => {
                self.consume();
                match self.iter.next() {
                    Some('w') => Some(statelist.characters(character_sets::letters())),
                    Some('W') => Some(statelist.non_characters(character_sets::letters())),
                    Some('d') => Some(statelist.characters(character_sets::digits())),
                    Some('D') => Some(statelist.non_characters(character_sets::digits())),
                    Some('s') => Some(statelist.characters(character_sets::whitespace())),
                    Some('S') => Some(statelist.non_characters(character_sets::whitespace())),
                    Some('*') => Some(statelist.character('*')),
                    Some('+') => Some(statelist.character('+')),
                    Some('\\') => Some(statelist.character('\\')),
                    Some('(') => Some(statelist.character('(')),
                    Some(')') => Some(statelist.character(')')),
                    Some('.') => Some(statelist.character('.')),
                    _ => {
                        self.error_cur();
                        None
                    }
                }
                // self.consume();
                // Some(statelist.characters(character_sets::letters()))
            }
            Some('[') => {
                let mut negate = false;
                let mut inclusive = HashSet::new();
                let mut exclusive = HashSet::new();
                if let Some('^') = self.iter.peek() {
                    self.iter.next();
                    negate = true;
                }

                loop {
                    match self.iter.next() {
                        Some(']') => break,
                        Some('\\') => match self.iter.next() {
                            Some(']') => {
                                inclusive.insert(']');
                            }
                            Some('\\') => {
                                inclusive.insert('\\');
                            }
                            Some('w') => inclusive.extend(character_sets::letters()),
                            Some('W') => exclusive.extend(character_sets::letters()),
                            Some('d') => inclusive.extend(character_sets::digits()),
                            Some('D') => exclusive.extend(character_sets::digits()),
                            Some('s') => inclusive.extend(character_sets::whitespace()),
                            Some('S') => exclusive.extend(character_sets::whitespace()),
                            _ => {
                                self.error_cur();
                                return None;
                            }
                        },
                        Some(c) if c.is_ascii() => {
                            if let Some('-') = self.iter.peek() {
                                self.iter.next();
                                match self.iter.next() {
                                    Some(high) if c.is_ascii() => {
                                        if let Ok(set) = character_sets::range(c as u8, high as u8)
                                        {
                                            inclusive.extend(set);
                                        } else {
                                            self.error_cur();
                                            return None;
                                        }
                                    }
                                    _ => {
                                        self.error_cur();
                                        return None;
                                    }
                                }
                            } else {
                                inclusive.insert(c);
                            }
                        }
                        Some(c) => {
                            inclusive.insert(c);
                        }
                        None => {
                            self.error_cur();
                            return None;
                        }
                    };
                }

                if negate {
                    Some(statelist.inclusive_exclusive_characters(exclusive, inclusive))
                } else {
                    Some(statelist.inclusive_exclusive_characters(inclusive, exclusive))
                }
            }
            Some(&c) => {
                self.consume();
                Some(statelist.character(c))
            }
            None => {
                self.error_next();
                None
            }
        }
    }

    fn consume(&mut self) -> Option<char> {
        self.index += 1;
        self.iter.next()
    }

    // TODO allow an optional error_next message be passed for better error_next reporting
    fn error_next(&mut self) {
        self.errors.push(self.index);
        self.iter.next();
    }

    fn error_cur(&mut self) {
        self.errors.push(self.index);
    }
}
