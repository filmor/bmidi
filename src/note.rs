extern crate pitch_calc;

use pitch_calc::Step;
use std::fmt;

#[derive(Clone)]
pub struct Note(u8);

impl Note {
    pub fn new(value: u8) -> Note {
        Note(value)
    }

    pub fn freq(&self) -> f32 {
        Step(self.0 as f32).to_hz().0
    }
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", Step(self.0 as f32).letter_octave())
    }
}
