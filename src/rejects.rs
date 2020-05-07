use crate::nfa::State;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Rejects {
    start: usize,
    statelist: Vec<State>,
}

#[allow(dead_code)]
impl Rejects {
    pub fn _new2(pat: &str) {}

    pub(crate) fn new(start: usize, statelist: Vec<State>) -> Rejects {
        Rejects { start, statelist }
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
            _ => {} // Match and Nil and InclusiveTransition and ExclusiveTransition don't have epsilon transitions
        }
    }
}
