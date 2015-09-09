use note::Note;

pub type Ticks = u32;
pub type Byte = u8;

#[derive(Debug,Clone,Copy)]
pub enum KeyEventType {
    Press,
    Release,
    Aftertouch
}

#[derive(Debug,Clone)]
pub enum EventType {
    Key { typ: KeyEventType, note: Note, velocity: Byte },
    ControlChange { controller: Byte, value: Byte },
    PatchChange { program: Byte },
    ChannelAftertouch { channel: Byte },
    PitchWheelChange { value: u16 }, // 14 relevant bits
    Meta { typ: Byte, data: Vec<u8> },
    // SysEx
}

#[derive(Debug,Clone)]
pub struct Event {
    pub delay: Ticks,
    pub channel: Byte,
    pub typ: EventType,
}

