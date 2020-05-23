use crate::nfa::{State, StateList};
use crate::parser;
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Rejects {
    start: usize,
    statelist: Vec<State>,
}

impl ToTokens for Rejects {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let start = self.start;
        let mut wrapper_stream = proc_macro2::TokenStream::new();
        wrapper_stream.append_all(quote! {
            let mut builder = rejects::builder::Builder::new(#start);
        });
        for state in self.statelist.iter() {
            wrapper_stream.append_all(quote! {
                builder.add_state(#state);
            });
        }
        tokens.append_all(quote! {
            {
                #wrapper_stream
                builder.build()
            }
        });
    }
}

#[allow(dead_code)]
impl Rejects {
    pub fn new(pat: &str) -> Result<Rejects, Vec<u32>> {
        let (start, statelist) = parser::parse(pat)?;
        Ok(Rejects { start, statelist })
    }

    pub(crate) fn from(start: usize, states: Vec<State>) -> Rejects {
        Rejects {
            start,
            statelist: states,
        }
    }

    /// returns index of the end of the match. Uses maximal munch.
    pub fn find_end(&self, s: &str) -> isize {
        let mut states = HashSet::new();
        states.insert(self.start);
        self.epsilon_transition(&mut states, self.start);
        let mut len = 0;

        for (i, c) in s.chars().enumerate() {
            let mut newstates = HashSet::new();
            for &state in states.iter() {
                self.character_transition(&mut newstates, state, c);
            }
            if newstates.len() == 0 {
                return (i as isize) - 1;
            } else {
                states = newstates;
            }
            len += 1;
        }
        let accept = states.into_iter().any(|n| {
            if let State::Match = &self.statelist[n] {
                true
            } else {
                false
            }
        });
        if accept {
            len - 1
        } else {
            -1
        }
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
            _ => {} // Match and Nil and Transition don't have epsilon transitions
        }
    }
}
