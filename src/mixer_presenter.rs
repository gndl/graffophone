
use talker::identifier::{Id, RIdentifier};

use session::mixer::RMixer;

use crate::output_presenter::OutputPresenter;

pub struct MixerPresenter {
    identifier: RIdentifier,
    outputs: Vec<OutputPresenter>,
}

impl MixerPresenter {
    pub fn new(mixer: &RMixer) -> MixerPresenter {
        let mxr = mixer.borrow();
        let identifier = mxr.identifier().clone();
        let mut outputs = Vec::with_capacity(mxr.outputs().len());

        for output in mxr.outputs() {
            outputs.push(OutputPresenter::from(output));
        }

        Self {
            identifier,
            outputs,
        }
    }

    pub fn id(&self) -> Id {
        self.identifier.borrow().id()
    }

    pub fn name(&self) -> String {
        self.identifier.borrow().name().to_string()
    }

    pub fn outputs<'a>(&'a self) -> &'a Vec<OutputPresenter> {
        &self.outputs
    }

    pub fn mutable_outputs<'a>(&'a mut self) -> &'a mut Vec<OutputPresenter> {
        &mut self.outputs
    }

    pub fn add_output(&mut self, output: OutputPresenter) {
        self.outputs.push(output);
    }

    pub fn remove_output(&mut self, id: Id) {

        for idx in 0..self.outputs.len() {
            if self.outputs[idx].identifier().id() == id {
                self.outputs.remove(idx);
                break;
            }
        }
    }
}