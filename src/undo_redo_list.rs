
pub struct UndoRedoList {
    states: Vec<String>,
    position: usize,
}

impl UndoRedoList {
    pub fn new() -> UndoRedoList {
        let mut states = Vec::with_capacity(1024);
        states.push("".to_string());

        Self {
            states,
            position: 0,
        }
    }

    pub fn reset(&mut self) {
        self.states.clear();
        self.states.push("".to_string());
        self.position = 0;
    }

    pub fn new_state(&mut self, state: String) {

        let mut end_pos = self.states.len() - 1;

        if self.position < end_pos {
            let mut start_pos = self.position;
            let step_count = (end_pos - start_pos + 1) / 2;

            for _ in 0..step_count {
                self.states.swap(start_pos, end_pos);
                start_pos += 1;
                end_pos -= 1;
            }
        }
        self.states.push(state);
        self.position = self.states.len() - 1;
    }

    pub fn undo(&mut self) -> Option<String> {
        if self.position > 0 {
            self.position -= 1;
            Some(self.states[self.position].clone())
        }
        else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<String> {
        if self.position < self.states.len() - 1 {
            self.position += 1;
            Some(self.states[self.position].clone())
        }
        else {
            None
        }
    }
}
