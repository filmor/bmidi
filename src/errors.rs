#[derive(Debug)]
pub enum MidiError {
    EndOfStream,
    InvalidUtf8String,
    InvalidHeader,
    UnsupportedVersion,
}
