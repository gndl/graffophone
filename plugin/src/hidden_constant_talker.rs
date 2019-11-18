use crate::talker::{Talker, TalkerBase};
use crate::voice::Voice;

pub struct HiddenConstantTalker {
    base: TalkerBase,
    value: f32,
}

impl HiddenConstantTalker {
    pub fn new(value: Option<f32>) -> HiddenConstantTalker {
        Self {
            base: TalkerBase::new(),
            value: value.unwrap_or(1.),
        }
    }
}

impl Talker for HiddenConstantTalker {
    fn base<'b>(&'b self) -> &'b TalkerBase {
        &self.base
    }
    fn depends_of(&self, _id: u32) -> bool {
        true
    }
}
/*
and ?(value = 1.) () =
  object(self) inherit c
    val mutable mOutput = None

    method provideOutput =
      match mOutput with
      | Some o -> o
      | None ->
        let o = {vTag = defOutputTag; port = 0; tick = 0; len = 1;
                 tkr = self#base; cor = Cornet.init ~v:value 1}
        in
        mOutput <- Some o;
        o


    method! getValue = Float (Voice.get self#provideOutput 0)


    method! setValue v =
      let output = self#provideOutput in
      fill output 0 (dim output) (v2f v)

    method getKind = "hiddenConstant"
    method! isHidden = true
    method! getVoices = [|self#provideOutput|]

    method! talk _ tick len =
      let output = self#provideOutput in

      if output.len < len then (
        output.cor <- Cornet.init ~v:(Cornet.get output.cor 0) len;
        output.len <- len;
      );
      output.tick <- tick;
  end
*/
