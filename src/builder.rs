use crate::nfa::State;
use crate::rejects::Rejects;
use std::collections::HashSet;

pub struct Builder {
    start: usize,
    statelist: Vec<State>,
}

impl Builder {
    pub fn new(start: usize) -> Builder {
        Builder {
            start,
            statelist: Vec::new(),
        }
    }

    pub fn build(self) -> Rejects {
        Rejects::from(self.start, self.statelist)
    }

    pub fn add_state(&mut self, state: State) -> &Builder {
        self.statelist.push(state);
        self
    }

    pub fn with_transition(
        &mut self,
        inclusive: HashSet<char>,
        exclusive: HashSet<char>,
        out: Option<usize>,
    ) -> &Builder {
        self.statelist.push(State::Transition {
            inclusive,
            exclusive,
            out,
        });
        self
    }

    pub fn with_split(&mut self, out1: usize, out2: Option<usize>) -> &Builder {
        self.statelist.push(State::Split { out1, out2 });
        self
    }

    pub fn with_match(&mut self) -> &Builder {
        self.statelist.push(State::Match);
        self
    }

    pub fn with_nil(&mut self) -> &Builder {
        self.statelist.push(State::Nil);
        self
    }
}
