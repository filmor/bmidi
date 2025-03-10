use std::io::Read;

use crate::note::Note;

use crate::types::EventType::*;
use crate::types::KeyEventType::*;
use crate::types::*;

use crate::errors::*;

#[derive(Default, Clone, Copy)]
struct Status {
    channel: Byte,
    opcode: Byte,
}

pub struct MidiRead<R>
where
    R: Read,
{
    reader: R,
    running_status: Status,
}

impl<R> MidiRead<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            running_status: Status::default(),
        }
    }

    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError> {
        match self.reader.read(output).ok() {
            Some(len) if len == output.len() => Ok(()),
            _ => Err(MidiError::EndOfStream),
        }
    }

    pub fn read_bytes_dyn(&mut self, length: usize) -> Result<Vec<u8>, MidiError> {
        let mut res = vec![0u8; length];
        self.read(&mut res)?;
        Ok(res)
    }

    pub fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], MidiError> {
        let mut res = [0u8; N];
        self.read(&mut res)?;
        Ok(res)
    }

    pub fn read_string(&mut self, length: usize) -> Result<String, MidiError> {
        let res = self.read_bytes_dyn(length)?;
        String::from_utf8(res).or_else(|_| Err(MidiError::InvalidUtf8String))
    }

    pub fn read_byte(&mut self) -> Result<u8, MidiError> {
        let res = self.read_bytes::<1>()?;
        Ok(res[0])
    }

    pub fn read_short(&mut self) -> Result<u16, MidiError> {
        let res = self.read_bytes::<2>()?;
        Ok(u16::from(res[0]) << 8 | u16::from(res[1]))
    }

    pub fn read_int(&mut self) -> Result<u32, MidiError> {
        let res = self.read_bytes::<4>()?;
        Ok(
            (((res[0] as u32) << 8 | (res[1] as u32)) << 8 | (res[2] as u32)) << 8
                | (res[3] as u32),
        )
    }

    pub fn read_var_len(&mut self) -> Result<u32, MidiError> {
        let mut res = 0;

        loop {
            let next_byte = u32::from(self.read_byte()?);
            res <<= 7;
            res |= next_byte & 0x7f;
            if next_byte & 0x80 == 0 {
                break;
            }
        }

        Ok(res)
    }
}

impl<R: Read> Iterator for MidiRead<R> {
    // type Item = Result<Event, MidiError>;
    type Item = Event;

    fn next(&mut self) -> Option<Event> /* Option<Result<Event, MidiError>>*/ {
        // TODO: Break cleanly if self.reader is exhausted
        let ticks = self.read_var_len().ok()?;

        let mut first_byte = self.read_byte().ok()?;

        if (first_byte & 1 << 7) != 0 {
            let status_byte = first_byte;
            first_byte = self.read_byte().ok()?;
            self.running_status = Status {
                channel: status_byte & 0xf,
                opcode: (status_byte & 0xf0) >> 4,
            };
        }

        let status = self.running_status;

        let event_type = match status.opcode {
            0x8..=0xa => {
                let note = Note::new(first_byte);
                let velocity = self.read_byte().ok()?;

                let typ = if status.opcode == 0x8 || (status.opcode == 0x9 && velocity == 0) {
                    Release
                } else if status.opcode == 0x9 {
                    Press
                } else {
                    Aftertouch
                };

                Key {
                    typ,
                    note,
                    velocity,
                }
            }
            0xb => ControlChange {
                controller: first_byte,
                value: self.read_byte().ok()?,
            },
            0xc => PatchChange {
                program: first_byte,
            },
            0xd => ChannelAftertouch {
                channel: first_byte,
            },
            0xe => PitchWheelChange {
                value: ((first_byte as u16) << 7) | self.read_byte().ok()? as u16,
            },
            0xf => {
                if status.channel == 0xf {
                    let typ = first_byte;

                    if typ == 0x2f {
                        // End-of-track
                        let null_byte = self.read_byte().ok()?;
                        assert!(null_byte == 0u8);
                        return None;
                    }

                    let length = self.read_var_len().ok()? as usize;
                    let data = self.read_bytes_dyn(length).ok()?;

                    Meta { typ, data }
                } else {
                    panic!("Nope")
                }
            }
            _ => unreachable!("Invalid opcode"),
        };

        let event = Event {
            delay: ticks,
            channel: status.channel,
            typ: event_type,
        };

        Some(event)
    }
}
