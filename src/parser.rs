use std::iter::Peekable;
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
#[derive(Debug)]
pub struct Fragment {
    start: i32,
    endstates: Vec<i32>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum State {
    Transition { chars: Vec<char>, out: i32 }, // TODO change vec to hashset
    NonTransition { chars: Vec<char>, out: i32 },
    Split { out1: i32, out2: i32 },
    Match,
    Nil,
}

#[allow(dead_code)]
impl State {
    fn make_transition(chars: Vec<char>) -> State {
        State::Transition { chars, out: -1 }
    }

    fn make_nontransition(chars: Vec<char>) -> State {
        State::NonTransition { chars, out: -1 }
    }

    fn make_split() -> State {
        State::Split { out1: -1, out2: -1 }
    }

    fn make_match() -> State {
        State::Match
    }

    fn make_nil() -> State {
        State::Nil
    }

    fn set_out(&mut self, newout: i32) {
        match self {
            State::Transition {
                chars: _,
                ref mut out,
            } => *out = newout,
            State::NonTransition {
                chars: _,
                ref mut out,
            } => *out = newout,
            State::Split {
                out1: _,
                ref mut out2,
            } => *out2 = newout,
            _ => {} // State::Match and State::Nil but this shouldn't be reached
        }
    }
}

#[derive(Debug)]
pub struct Regex {
    states: Vec<State>,
    start: i32,
}

impl Regex {
    pub fn new() -> Regex {
        Regex {
            start: -1,
            states: Vec::new(),
        }
    }

    pub fn find(&self, s: &str) -> bool {
        let mut cur = self.start;
        for c in s.chars() {
            self.step(&mut cur, c);
            if cur == -1 {
                return false;
            }
        }
        if let State::Match = self.states[cur as usize] {
            return true;
        }
        false
    }

    fn step(&self, cur: &mut i32, c: char) {
        let state = &self.states[*cur as usize];
        match state {
            State::Transition { chars, out } => {
                if chars.contains(&c) {
                    *cur = *out;
                } else {
                    *cur = -1;
                }
            }
            _ => {}
        }
    }

    fn union(&mut self, f1: Option<Fragment>, _f2: Option<Fragment>) -> Option<Fragment> {
        f1
    }

    fn concatenation(
        &mut self,
        f1opt: Option<Fragment>,
        f2opt: Option<Fragment>,
    ) -> Option<Fragment> {
        let f1 = match f1opt {
            Some(f) => f,
            None => return None,
        };
        let f2 = match f2opt {
            Some(f) => f,
            None => return Some(f1),
        };

        for &dangler in f1.endstates.iter() {
            self.link(dangler, f2.start);
        }

        Some(Fragment {
            start: f1.start,
            endstates: f2.endstates,
        })
    }

    fn unary_operator(&mut self, f: Option<Fragment>, op: Option<char>) -> Option<Fragment> {
        if let Some(frag) = f {
            match op {
                Some('*') => Some(self.kleene(frag)),
                Some('?') => Some(self.question_mark(frag)),
                Some('+') => Some(self.plus(frag)),
                _ => Some(frag),
            }
        } else {
            None
        }
    }

    fn kleene(&mut self, f: Fragment) -> Fragment {
        f
    }

    fn question_mark(&mut self, f: Fragment) -> Fragment {
        f
    }

    fn plus(&mut self, f: Fragment) -> Fragment {
        f
    }

    fn character(&mut self, c: char) -> Fragment {
        self.characters(vec![c])
    }

    fn characters(&mut self, chars: Vec<char>) -> Fragment {
        let state = self.add_state(State::make_transition(chars));
        Fragment {
            start: state,
            endstates: vec![state],
        }
    }

    fn add_state(&mut self, state: State) -> i32 {
        if self.states.len() == std::i32::MAX as usize {
            // TODO panic
        }
        self.states.push(state);
        self.states.len() as i32 - 1
    }

    fn link(&mut self, from: i32, to: i32) {
        &self.states[from as usize].set_out(to);
    }
}

#[allow(dead_code)]
pub struct Parser<'a> {
    iter: Peekable<Chars<'a>>,
    index: u32,
    errors: Vec<u32>,
}

impl<'a> Parser<'a> {
    pub fn parse(s: &str) -> Result<Regex, Vec<u32>> {
        let mut parser = Parser::new(s);
        let mut regex = Regex::new();
        if let Some(frag) = parser.parse_union(&mut regex) {
            let match_state = regex.add_state(State::make_match());
            regex.start = frag.start;
            for &dangler in frag.endstates.iter() {
                regex.link(dangler, match_state);
            }
        }
        // ensure we are at the end of the string
        if let Some(_) = parser.iter.next() {
            parser.error();
        }
        if parser.errors.len() > 0 {
            return Err(parser.errors);
        }
        Ok(regex)
    }

    fn new<'b: 'a>(s: &'b str) -> Parser<'a> {
        Parser {
            iter: s.chars().peekable(),
            index: 0,
            errors: Vec::new(),
        }
    }

    fn parse_union(&mut self, re: &mut Regex) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                let l = self.parse_concat(re);
                let r = self.parse_union_prime(re);
                re.union(l, r)
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some(_) => {
                let l = self.parse_concat(re);
                let r = self.parse_union_prime(re);
                re.union(l, r)
            }
            None => self.error(),
        }
    }

    fn parse_union_prime(&mut self, re: &mut Regex) -> Option<Fragment> {
        match self.iter.peek() {
            Some(')') => None,
            Some('|') => {
                self.consume();
                self.parse_union(re)
            }
            Some(_) => self.error(),
            None => None,
        }
    }

    fn parse_concat(&mut self, re: &mut Regex) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                let l = self.parse_unary(re);
                let r = self.parse_concat_prime(re);
                re.concatenation(l, r)
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some(_) => {
                let l = self.parse_unary(re);
                let r = self.parse_concat_prime(re);
                re.concatenation(l, r)
            }
            None => self.error(),
        }
    }

    fn parse_concat_prime(&mut self, re: &mut Regex) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => self.parse_concat(re),
            Some(')') => None,
            Some('*') | Some('?') | Some('+') => self.error(),
            Some('|') => None,
            Some(_) => self.parse_concat(re),
            None => None,
        }
    }

    fn parse_unary(&mut self, re: &mut Regex) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                let l = self.parse_paren(re);
                let r = self.parse_unaryop();
                re.unary_operator(l, r)
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') => self.error(),
            Some(_) => {
                let l = self.parse_paren(re);
                let r = self.parse_unaryop();
                re.unary_operator(l, r)
            }
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

    fn parse_paren(&mut self, re: &mut Regex) -> Option<Fragment> {
        match self.iter.peek() {
            Some('(') => {
                self.consume();
                let fragment = self.parse_union(re);
                if let Some(')') = self.iter.peek() {
                    self.consume();
                    fragment
                } else {
                    self.error()
                }
            }
            Some(')') | Some('*') | Some('?') | Some('+') | Some('|') | None => self.error(),
            Some(_) => self.parse_term(re),
        }
    }

    fn parse_term(&mut self, re: &mut Regex) -> Option<Fragment> {
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
                Some(re.character(c))
            }
            None => self.error(),
        }
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
            assert!(
                Parser::parse(regex).is_ok(),
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
            assert!(
                Parser::parse(regex).is_err(),
                r#""{}" should be recognized as an invalid regex"#,
                regex
            );
        }
    }
}
