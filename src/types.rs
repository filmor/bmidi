extern crate pitch_calc;

use pitch_calc::Step;
use std::fmt;

pub type Ticks = u32;
pub type Byte = u8;

pub struct Note(u8);

impl Note {
    pub fn new(value: u8) -> Note {
        Note(value)
    }
}

impl fmt::Debug for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", Step(self.0 as f32).letter_octave())
    }
}

#[derive(Debug)]
pub enum KeyEventType {
    Press,
    Release,
    Aftertouch
}

#[derive(Debug)]
pub enum EventType {
    Key { typ: KeyEventType, note: Note, velocity: Byte },
    ControlChange { controller: Byte, value: Byte },
    PatchChange { program: Byte },
    ChannelAftertouch { channel: Byte },
    PitchWheelChange { value: u16 }, // 14 relevant bits
    Meta { typ: Byte, data: Vec<u8> },
    // SysEx
}

#[derive(Debug)]
pub struct Event {
    pub delay: Ticks,
    pub channel: Byte,
    pub typ: EventType,
}

pub type Track = Vec<Event>;

pub struct File {
    pub tracks: Vec<Track>,
    pub format: u16,
    pub division: Ticks,
}
