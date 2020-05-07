use std::collections::HashSet;
use std::iter::Peekable;
use std::ops::Index;
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
    start: usize,
    endstates: Vec<usize>,
}

mod character_classes {
    use std::collections::HashSet;

    pub(super) fn letters() -> HashSet<char> {
        let mut set = HashSet::new();
        for c in b'a'..=b'z' {
            set.insert(c as char);
        }
        for c in b'A'..=b'Z' {
            set.insert(c as char);
        }
        set
    }

    pub(super) fn digits() -> HashSet<char> {
        let mut set = HashSet::new();
        for c in b'0'..=b'9' {
            set.insert(c as char);
        }
        set
    }

    pub(super) fn whitespace() -> HashSet<char> {
        let mut set = HashSet::new();
        set.insert(' ');
        set.insert('\t');
        set
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum State {
    Transition {
        inclusive: HashSet<char>,
        exclusive: HashSet<char>,
        out: Option<usize>,
    },
    Split {
        out1: usize,
        out2: Option<usize>,
    },
    Match,
    Nil,
}

#[allow(dead_code)]
impl State {
    fn make_inclusive_transition(chars: HashSet<char>) -> State {
        State::Transition {
            inclusive: chars,
            exclusive: HashSet::new(),
            out: None,
        }
    }

    fn make_exclusive_transition(chars: HashSet<char>) -> State {
        State::Transition {
            inclusive: HashSet::new(),
            exclusive: chars,
            out: None,
        }
    }

    fn make_split(out1: usize, out2: Option<usize>) -> State {
        State::Split { out1, out2: out2 }
    }

    fn make_match() -> State {
        State::Match
    }

    fn make_nil() -> State {
        State::Nil
    }

    fn set_out(&mut self, newout: usize) {
        match self {
            State::Transition {
                inclusive: _,
                exclusive: _,
                ref mut out,
            } => *out = Some(newout),
            State::Split {
                out1: _,
                ref mut out2,
            } => *out2 = Some(newout),
            _ => {} // State::Match and State::Nil but this shouldn't be reached
        }
    }

    fn transition(&self, c: char) -> Option<usize> {
        match self {
            State::Transition {
                inclusive,
                exclusive,
                ref out,
            } => {
                if (inclusive.len() > 0 && inclusive.contains(&c))
                    || (exclusive.len() > 0 && !exclusive.contains(&c))
                {
                    *out
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct StateList {
    states: Vec<State>,
}

impl StateList {
    fn new() -> StateList {
        StateList { states: Vec::new() }
    }

    fn union(&mut self, f1opt: Option<Fragment>, f2opt: Option<Fragment>) -> Option<Fragment> {
        let mut f1 = f1opt?;
        let f2 = match f2opt {
            Some(f) => f,
            None => return Some(f1),
        };

        let start = self.add_state(State::make_split(f1.start, Some(f2.start)));
        f1.endstates.extend(f2.endstates);
        Some(Fragment {
            start,
            endstates: f1.endstates,
        })
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
                _ => Some(frag), // No operand so just return what we have
            }
        } else {
            None
        }
    }

    fn kleene(&mut self, f: Fragment) -> Fragment {
        let start = self.add_state(State::make_split(f.start, None));
        for &dangler in f.endstates.iter() {
            self.link(dangler, start);
        }
        Fragment {
            start,
            endstates: vec![start],
        }
    }

    fn question_mark(&mut self, f: Fragment) -> Fragment {
        let start = self.add_state(State::make_split(f.start, None));
        let mut endstates = vec![start];
        endstates.extend(f.endstates);
        Fragment { start, endstates }
    }

    fn plus(&mut self, f: Fragment) -> Fragment {
        let splitter = self.add_state(State::make_split(f.start, None));
        for &dangler in f.endstates.iter() {
            self.link(dangler, splitter);
        }
        Fragment {
            start: f.start,
            endstates: vec![splitter],
        }
    }

    fn character(&mut self, c: char) -> Fragment {
        let mut set = HashSet::new();
        set.insert(c);
        self.characters(set)
    }

    fn characters(&mut self, chars: HashSet<char>) -> Fragment {
        let state = self.add_state(State::make_inclusive_transition(chars));
        Fragment {
            start: state,
            endstates: vec![state],
        }
    }

    fn non_characters(&mut self, chars: HashSet<char>) -> Fragment {
        let state = self.add_state(State::make_exclusive_transition(chars));
        Fragment {
            start: state,
            endstates: vec![state],
        }
    }

    fn add_state(&mut self, state: State) -> usize {
        self.states.push(state);
        self.states.len() - 1
    }

    fn link(&mut self, from: usize, to: usize) {
        &self.states[from].set_out(to);
    }
}

impl Index<usize> for StateList {
    type Output = State;

    fn index(&self, n: usize) -> &State {
        &self.states[n]
    }
}

#[derive(Debug)]
pub struct Regex {
    start: usize,
    statelist: StateList,
}

#[allow(dead_code)]
impl Regex {
    fn new(start: usize, statelist: StateList) -> Regex {
        Regex { start, statelist }
    }

    pub fn find(&self, s: &str) -> bool {
        let mut states = HashSet::new();
        states.insert(self.start);
        self.epsilon_transition(&mut states, self.start);

        for c in s.chars() {
            let mut newstates = HashSet::new();
            for &state in states.iter() {
                self.character_transition(&mut newstates, state, c);
            }
            if newstates.len() == 0 {
                return false;
            } else {
                states = newstates;
            }
        }
        states.into_iter().any(|n| {
            if let State::Match = &self.statelist[n] {
                true
            } else {
                false
            }
        })
    }

    fn character_transition(&self, newstates: &mut HashSet<usize>, state: usize, symbol: char) {
        if let Some(out) = &self.statelist[state].transition(symbol) {
            newstates.insert(*out);
            self.epsilon_transition(newstates, *out);
        }
    }

    fn epsilon_transition(&self, newstates: &mut HashSet<usize>, state: usize) {
        match &self.statelist[state] {
            State::Split { out1, out2 } => {
                newstates.insert(*out1);
                self.epsilon_transition(newstates, *out1);
                if let Some(out) = *out2 {
                    newstates.insert(out);
                    self.epsilon_transition(newstates, out);
                }
            }
            _ => {} // TODO Match and Nil and InclusiveTransition and ExclusiveTransition
        }
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
            return Ok(Regex::new(frag.start, statelist));
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
            Some('\\') => {
                self.consume();
                match self.iter.next() {
                    Some('w') => Some(statelist.characters(character_classes::letters())),
                    Some('W') => Some(statelist.non_characters(character_classes::letters())),
                    Some('d') => Some(statelist.characters(character_classes::digits())),
                    Some('D') => Some(statelist.non_characters(character_classes::digits())),
                    Some('s') => Some(statelist.characters(character_classes::whitespace())),
                    Some('S') => Some(statelist.non_characters(character_classes::whitespace())),
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
                // Some(statelist.characters(character_classes::letters()))
            }
            Some('[') => {
                let mut negate = false;
                let mut chars = HashSet::new();
                if let Some('^') = self.iter.peek() {
                    self.iter.next();
                    negate = true;
                }

                loop {
                    match self.iter.next() {
                        Some(']') => break,
                        Some('\\') => match self.iter.next() {
                            Some(']') => chars.insert(']'),
                            Some('\\') => chars.insert('\\'),
                            _ => {
                                self.error_cur();
                                return None;
                            }
                        },
                        Some(c) => chars.insert(c),
                        // TODO handle escapes codes
                        None => {
                            self.error_cur();
                            return None;
                        }
                    };
                }

                if negate {
                    Some(statelist.non_characters(chars))
                } else {
                    Some(statelist.characters(chars))
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
            // r"(\w)",
            // r"\\",
        ];
        for regex in regexes {
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

    #[test]
    fn test_union() {}

    #[test]
    fn test_concatenation() {}
}
