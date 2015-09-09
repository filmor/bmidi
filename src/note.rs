extern crate pitch_calc;

use pitch_calc::{Hz, Step};
use std::fmt;
use std::convert::{From};

#[derive(Copy, Clone)]
pub struct Note(u8);

impl Note {
    pub fn new(value: u8) -> Note {
        Note(value)
    }

    pub fn to_step(&self) -> Step {
        Step(self.0 as f32)
    }
}

impl From<Note> for Hz {
    fn from(note: Note) -> Hz {
        note.to_step().to_hz()
    }
}

impl From<Note> for Step {
    fn from(note: Note) -> Step {
        note.to_step()
    }
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.to_step().letter_octave())
    }
}
