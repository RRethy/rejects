use std::cell::RefCell;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::Chars;

/// LL(1) CFG for the supported regular expression syntax.
/// https://smlweb.cpsc.ucalgary.ca/vital-stats.php?grammar=UNION+++++-%3E+CONCAT%0D%0A+++++++++++%7C+UNION+cup+CONCAT.%0D%0ACONCAT++++-%3E+UNARY%0D%0A+++++++++++%7C+CONCAT+dot+UNARY.%0D%0AUNARY+++++-%3E+PAREN+UNARYOP.%0D%0AUNARYOP+++-%3E+*%0D%0A+++++++++++%7C+%3F%0D%0A+++++++++++%7C+%2B%0D%0A+++++++++++%7C.%0D%0APAREN+++++-%3E+TERM%0D%0A+++++++++++%7C+%28+UNION+%29.%0D%0ATERM++++++-%3E+terminal.%0D%0A
/// grammar before left-recursion is removed (we don't include $ in the grammar because it's just a
/// simple check after parsing)
/*
UNION     -> CONCAT
| UNION cup CONCAT.
CONCAT    -> UNARY
| CONCAT dot UNARY.
UNARY     -> PAREN UNARYOP.
UNARYOP   -> *
| ?
| +
|.
PAREN     -> TERM
| ( UNION ).
TERM      -> terminal.
*/
/// Same grammar as above but with left-recursion removed so it is LL(1)
/*
UNION →	CONCAT UNION1 .
UNION1 →	cup CONCAT UNION1
|	.
CONCAT →	UNARY CONCAT1 .
CONCAT1 →	dot UNARY CONCAT1
|	.
UNARY →	PAREN UNARYOP .
UNARYOP →	*
|	?
|	+
|	.
PAREN →	TERM
|	( UNION ) .
TERM →	terminal .
*/

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
enum State {
    Transition {
        chars: Vec<char>,
        out: Rc<RefCell<State>>,
    },
    NonTransition {
        chars: Vec<char>,
        out: Rc<RefCell<State>>,
    },
    Split {
        out1: Rc<RefCell<State>>,
        out2: Rc<RefCell<State>>,
    },
    Match,
    Nil,
}

impl State {
    fn make_nil() -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State::Nil))
    }

    fn make_transition(chars: Vec<char>) -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State::Transition {
            chars,
            out: State::make_nil(),
        }))
    }

    fn _make_nontransition(chars: Vec<char>) -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State::NonTransition {
            chars,
            out: State::make_nil(),
        }))
    }

    fn make_split(out1: Rc<RefCell<State>>, out2: Rc<RefCell<State>>) -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State::Split { out1, out2 }))
    }

    fn make_match() -> Rc<RefCell<State>> {
        Rc::new(RefCell::new(State::Match))
    }

    fn point_to(&mut self, to: Rc<RefCell<State>>) {
        match self {
            State::Transition {
                chars: _,
                ref mut out,
            } => *out = to,
            State::NonTransition {
                chars: _,
                ref mut out,
            } => *out = to,
            State::Split {
                out1: _,
                ref mut out2,
            } => *out2 = to,
            _ => {}
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Fragment {
    start: Rc<RefCell<State>>,
    danglers: Vec<Rc<RefCell<State>>>,
}

impl Fragment {
    pub fn matches(&self, _s: &str) -> bool {
        true
    }
}

#[allow(dead_code)]
pub struct Parser<'a> {
    iter: Peekable<Chars<'a>>,
    index: u32,
    errors: Vec<u32>,
}

impl<'a> Parser<'a> {
    pub fn parse(s: &str) -> Option<Fragment> {
        let mut parser = Parser::new(s);
        // ensure we are at the end of the string
        if let Some(_) = parser.iter.next() {
            parser.error();
        }
        if parser.errors.len() > 0 {
            return None;
        }
        if let Some(frag) = parser.parse_union() {
            let m = State::make_match();
            for dangler_ref in frag.danglers.iter() {
                let ref mut dangler = *dangler_ref.borrow_mut();
                dangler.point_to(Rc::clone(&m));
            }
            Some(frag)
        } else {
            None
        }
    }

    fn new<'b: 'a>(s: &'b str) -> Parser<'a> {
        Parser {
            iter: s.chars().peekable(),
            index: 0,
            errors: Vec::new(),
        }
    }

    fn parse_union(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => Self::union(self.parse_concat(), self.parse_union_prime()),
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some(_) => Self::union(self.parse_concat(), self.parse_union_prime()),
            None => self.error(),
        }
    }

    fn parse_union_prime(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some(')') => None,
            Some('|') => {
                self.consume();
                self.parse_union()
            }
            Some(_) => self.error(),
            None => None,
        }
    }

    fn parse_concat(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => Self::concatenation(self.parse_unary(), self.parse_concat_prime()),
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some(_) => Self::concatenation(self.parse_unary(), self.parse_concat_prime()),
            None => self.error(),
        }
    }

    fn parse_concat_prime(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => self.parse_concat(),
            Some(')') => None,
            Some('*') | Some('?') | Some('+') => self.error(),
            Some('|') => None,
            Some(_) => self.parse_concat(),
            None => None,
        }
    }

    fn parse_unary(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => Self::unary_operator(self.parse_paren(), self.parse_unaryop()),
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some(_) => Self::unary_operator(self.parse_paren(), self.parse_unaryop()),
            None => self.error(),
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

    fn parse_paren(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                self.consume();
                let fragment = self.parse_union();
                if let Some(')') = self.iter.peek() {
                    self.consume();
                    fragment
                } else {
                    self.error()
                }
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') | None => self.error(),
            Some(_) => self.parse_term(),
        }
    }

    fn parse_term(&mut self) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') | Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some('\\') => {
                self.consume();
                // TODO
                match self.iter.peek() {
                    Some('w') => {}
                    Some('W') => {}
                    Some('d') => {}
                    Some('D') => {}
                    Some('s') => {}
                    Some('S') => {}
                    Some('*') => {}
                    Some('+') => {}
                    Some('\\') => {}
                    Some('(') => {}
                    Some(')') => {}
                    Some('.') => {}
                    _ => return self.error(),
                }
                self.consume();
                None // TODO return a fragment
            }
            Some('[') => {
                // TODO unimplemented
                None
            }
            Some(&c) => {
                self.consume();
                Some(Self::character(c))
            }
            None => self.error(),
        }
    }

    fn union(f1: Option<Fragment>, _f2: Option<Fragment>) -> Option<Fragment> {
        f1
    }

    fn concatenation(f1: Option<Fragment>, _f2: Option<Fragment>) -> Option<Fragment> {
        f1
    }

    fn unary_operator(f: Option<Fragment>, op: Option<char>) -> Option<Fragment> {
        if let Some(frag) = f {
            match op {
                Some('*') => Some(Self::kleene(frag)),
                Some('?') => Some(Self::question_mark(frag)),
                Some('+') => Some(Self::plus(frag)),
                _ => None,
            }
        } else {
            None
        }
    }

    fn kleene(f: Fragment) -> Fragment {
        let start = State::make_split(Rc::clone(&f.start), State::make_nil());
        for dangler_ref in f.danglers.iter() {
            let ref mut dangler = *dangler_ref.borrow_mut();
            dangler.point_to(Rc::clone(&start));
        }
        let danglers = vec![Rc::clone(&start)];
        Fragment { start, danglers }
    }

    fn question_mark(f: Fragment) -> Fragment {
        f
    }

    fn plus(f: Fragment) -> Fragment {
        f
    }

    fn character(c: char) -> Fragment {
        Self::characters(vec![c])
    }

    fn characters(chars: Vec<char>) -> Fragment {
        let start = State::make_transition(chars);
        let danglers = vec![Rc::clone(&start)];
        Fragment { start, danglers }
    }

    fn consume(&mut self) -> Option<char> {
        self.index += 1;
        self.iter.next()
    }

    // TODO allow an optional error message be passed for better error reporting
    fn error(&mut self) -> Option<Fragment> {
        self.errors.push(self.index);
        self.iter.next();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_regexes() {
        let regexes = vec![
            r"abcd",
            r"a|b|c|d",
            r"(ab)|(cd)*",
            r"(a|b+c?|d)",
            r"(abcd)",
            r"a|a",
            r"(bc)",
            r"(abc)(abc)(abc)|(abc)(abc)",
            r"a|b+(c?|d)",
            r"(a|b)",
            r"(\w)",
            r"\\",
        ];
        for regex in regexes.iter() {
            assert_ne!(
                Parser::parse(regex),
                None,
                "\"{}\" should be recognized as valid regex",
                regex
            );
        }
    }

    #[test]
    fn test_invalid_regexes() {
        let regexes = vec![
            r"(abcd",
            r"a||c|d",
            r"|",
            r"()",
            r")aaab(",
            r"a|b+c?|d)",
            r"a)",
            r"(abcd)(",
            r"(a|)",
            r"(\a)",
            r"\",
            r"\\\",
        ];
        for regex in regexes.iter() {
            assert_eq!(
                Parser::parse(regex),
                None,
                r#""{}" should be recognized as an invalid regex"#,
                regex
            );
        }
    }
}
