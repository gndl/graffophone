
pub struct UndoRedoList {
    states: Vec<String>,
    position: usize,
}

impl UndoRedoList {
    pub fn new(init_state: String) -> UndoRedoList {
        let mut states = Vec::with_capacity(1024);
        states.push(init_state);

        Self {
            states,
            position: 0,
        }
    }

    pub fn reset(&mut self) {
        self.states.clear();
        self.position = 0;
    }

    pub fn new_state(&mut self, state: String) {

        let mut end_pos = self.end_position();

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
        self.position = self.end_position();
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
        if self.position < self.end_position() {
            self.position += 1;
            Some(self.states[self.position].clone())
        }
        else {
            None
        }
    }

    fn end_position(&self) -> usize {
        self.states.len().max(1) - 1
    }
}
