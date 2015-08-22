pub type Ticks = u32;
pub type Byte = u8;
pub type Note = u8;

pub enum KeyEventType {
    Press,
    Release,
    Aftertouch
}

pub enum EventType {
    Key { typ: KeyEventType, note: Note, velocity: Byte },
    ControlChange { controller: Byte, value: Byte },
    PatchChange { program: Byte },
    ChannelAftertouch { channel: Byte },
    PitchWheelChange { value: u16 }, // 14 relevant bits
    Meta { typ: Byte, data: Vec<u8> },
    SysEx
}

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
