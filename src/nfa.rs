use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use std::collections::HashSet;
use std::ops::Index;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct Fragment {
    pub(crate) start: usize,
    pub(crate) endstates: Vec<usize>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum State {
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
    pub fn make_transition(
        inclusive: HashSet<char>,
        exclusive: HashSet<char>,
        out: Option<usize>,
    ) -> State {
        let mut tran = State::make_inclusive_exclusive_transition(inclusive, exclusive);
        if let Some(c) = out {
            tran.set_out(c);
        }
        tran
    }

    pub fn make_split(out1: usize, out2: Option<usize>) -> State {
        State::Split { out1, out2 }
    }

    pub fn make_match() -> State {
        State::Match
    }

    pub fn make_nil() -> State {
        State::Nil
    }

    pub(crate) fn make_inclusive_exclusive_transition(
        inclusive: HashSet<char>,
        exclusive: HashSet<char>,
    ) -> State {
        State::Transition {
            inclusive,
            exclusive,
            out: None,
        }
    }

    pub(crate) fn make_inclusive_transition(chars: HashSet<char>) -> State {
        State::Transition {
            inclusive: chars,
            exclusive: HashSet::new(),
            out: None,
        }
    }

    pub(crate) fn make_exclusive_transition(chars: HashSet<char>) -> State {
        State::Transition {
            inclusive: HashSet::new(),
            exclusive: chars,
            out: None,
        }
    }

    pub(crate) fn set_out(&mut self, newout: usize) {
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

    pub(crate) fn transition(&self, c: char) -> Option<usize> {
        match self {
            State::Transition {
                inclusive,
                exclusive,
                ref out,
            } => {
                if (inclusive.len() > 0 && inclusive.contains(&c))
                    || (exclusive.len() > 0 && !exclusive.contains(&c))
                    || (inclusive.len() == 0 && exclusive.len() == 0)
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

impl ToTokens for State {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut wrapper_stream = proc_macro2::TokenStream::new();
        match self {
            State::Transition {
                inclusive,
                exclusive,
                out,
            } => {
                wrapper_stream.append_all(quote! {
                    let mut inclusive = std::collections::HashSet::new();
                    let mut exclusive = std::collections::HashSet::new();
                });
                for c in inclusive {
                    wrapper_stream.append_all(quote! {
                        inclusive.insert(#c);
                    });
                }
                for c in exclusive {
                    wrapper_stream.append_all(quote! {
                        exclusive.insert(#c);
                    });
                }
                match out {
                    Some(n) => wrapper_stream.append_all(quote! {
                        let out = Some(#n);
                    }),
                    None => wrapper_stream.append_all(quote! {
                        let out: Option<usize> = None;
                    }),
                }
                wrapper_stream.append_all(quote! {
                    let state = State::make_transition(inclusive, exclusive, out);
                });
            }
            State::Split { out1, out2 } => {
                match out2 {
                    Some(n) => wrapper_stream.append_all(quote! {
                        let out2 = Some(#n);
                    }),
                    None => wrapper_stream.append_all(quote! {
                        let out2: Option<usize> = None;
                    }),
                }
                wrapper_stream.append_all(quote! {
                    let state = State::make_split(#out1, out2);
                });
            }
            State::Match => {
                wrapper_stream.append_all(quote! {
                    let state = State::make_match();
                });
            }
            State::Nil => {
                wrapper_stream.append_all(quote! {
                    let state = State::make_nil();
                });
            }
        }
        tokens.append_all(quote! {
            {
                #wrapper_stream
                state
            }
        });
    }
}

#[derive(Debug)]
pub(crate) struct StateList {
    pub(crate) states: Vec<State>,
}

impl StateList {
    pub(crate) fn new() -> StateList {
        StateList { states: Vec::new() }
    }

    pub(crate) fn union(
        &mut self,
        f1opt: Option<Fragment>,
        f2opt: Option<Fragment>,
    ) -> Option<Fragment> {
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

    pub(crate) fn concatenation(
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

    pub(crate) fn unary_operator(
        &mut self,
        f: Option<Fragment>,
        op: Option<char>,
    ) -> Option<Fragment> {
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

    pub(crate) fn kleene(&mut self, f: Fragment) -> Fragment {
        let start = self.add_state(State::make_split(f.start, None));
        for &dangler in f.endstates.iter() {
            self.link(dangler, start);
        }
        Fragment {
            start,
            endstates: vec![start],
        }
    }

    pub(crate) fn question_mark(&mut self, f: Fragment) -> Fragment {
        let start = self.add_state(State::make_split(f.start, None));
        let mut endstates = vec![start];
        endstates.extend(f.endstates);
        Fragment { start, endstates }
    }

    pub(crate) fn plus(&mut self, f: Fragment) -> Fragment {
        let splitter = self.add_state(State::make_split(f.start, None));
        for &dangler in f.endstates.iter() {
            self.link(dangler, splitter);
        }
        Fragment {
            start: f.start,
            endstates: vec![splitter],
        }
    }

    pub(crate) fn character(&mut self, c: char) -> Fragment {
        let mut set = HashSet::new();
        set.insert(c);
        self.characters(set)
    }

    pub(crate) fn inclusive_exclusive_characters(
        &mut self,
        inclusive: HashSet<char>,
        exclusive: HashSet<char>,
    ) -> Fragment {
        let state = self.add_state(State::make_inclusive_exclusive_transition(
            inclusive, exclusive,
        ));
        Fragment {
            start: state,
            endstates: vec![state],
        }
    }

    pub(crate) fn characters(&mut self, chars: HashSet<char>) -> Fragment {
        let state = self.add_state(State::make_inclusive_transition(chars));
        Fragment {
            start: state,
            endstates: vec![state],
        }
    }

    pub(crate) fn non_characters(&mut self, chars: HashSet<char>) -> Fragment {
        let state = self.add_state(State::make_exclusive_transition(chars));
        Fragment {
            start: state,
            endstates: vec![state],
        }
    }

    pub(crate) fn add_state(&mut self, state: State) -> usize {
        self.states.push(state);
        self.states.len() - 1
    }

    pub(crate) fn link(&mut self, from: usize, to: usize) {
        &self.states[from].set_out(to);
    }
}

impl Index<usize> for StateList {
    type Output = State;

    fn index(&self, n: usize) -> &State {
        &self.states[n]
    }
}
