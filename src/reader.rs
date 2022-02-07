use crate::note::Note;

use crate::types::EventType::*;
use crate::types::KeyEventType::*;
use crate::types::*;

use crate::errors::*;

#[derive(Clone, Copy)]
struct Status {
    channel: Byte,
    opcode: Byte,
}

pub trait MidiRead {
    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError>;

    fn read_byte(&mut self) -> Result<u8, MidiError> {
        let mut res = [0u8];
        self.read(&mut res)?;
        Ok(res[0])
    }

    fn read_short(&mut self) -> Result<u16, MidiError> {
        let mut res = [0_u8; 2];
        self.read(&mut res)?;
        Ok(u16::from(res[0]) << 8 | u16::from(res[1]))
    }

    fn read_int(&mut self) -> Result<u32, MidiError> {
        let mut res = [0_u8; 4];
        self.read(&mut res)?;
        Ok(
            (((res[0] as u32) << 8 | (res[1] as u32)) << 8 | (res[2] as u32)) << 8
                | (res[3] as u32),
        )
    }

    fn read_var_len(&mut self) -> Result<u32, MidiError> {
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

impl<T> MidiRead for T
where
    T: Iterator<Item = u8>,
{
    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError> {
        for field in output.iter_mut() {
            match self.next() {
                Some(value) => *field = value,
                None => return Err(MidiError::EndOfStream),
            }
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, MidiError> {
        match self.next() {
            Some(value) => Ok(value),
            None => Err(MidiError::EndOfStream),
        }
    }
}

/* Disabled until inverse trait bounds are supported
 * Needs: use std::io::Read;

impl<T: Read + !Iterator<Item=u8>> MidiRead for T {
    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError> {
        match Read::read(self, output).ok() {
            Some(len) if len == output.len() => Ok(()),
            _ => Err(MidiError::EndOfStream)
        }
    }
}

*/

pub struct MidiReader<I: MidiRead> {
    reader: I,
    running_status: Status,
}

impl<I: MidiRead> MidiReader<I> {
    pub fn new(reader: I) -> MidiReader<I> {
        MidiReader {
            reader,
            running_status: Status {
                channel: 0,
                opcode: 0,
            },
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.reader.read_byte().unwrap()
    }

    pub fn read_int(&mut self) -> u32 {
        self.reader.read_int().unwrap()
    }

    pub fn read_short(&mut self) -> u16 {
        self.reader.read_short().unwrap()
    }

    pub fn read_bytes(&mut self, length: usize) -> Vec<u8> {
        let mut res = vec![0u8; length];
        self.reader.read(&mut res).unwrap();
        res
    }

    pub fn read_string(&mut self, length: usize) -> String {
        String::from_utf8(self.read_bytes(length)).unwrap()
    }
}

impl<I: MidiRead> Iterator for MidiReader<I> {
    // type Item = Result<Event, MidiError>;
    type Item = Event;

    fn next(&mut self) -> Option<Event> /* Option<Result<Event, MidiError>>*/ {
        // TODO: Break cleanly if self.reader is exhausted
        let ticks = self.reader.read_var_len().unwrap();

        let mut first_byte = self.read_byte();

        if (first_byte & 1 << 7) != 0 {
            let status_byte = first_byte;
            first_byte = self.read_byte();
            self.running_status = Status {
                channel: status_byte & 0xf,
                opcode: (status_byte & 0xf0) >> 4,
            };
        }

        let status = self.running_status;

        let event_type = match status.opcode {
            0x8 | 0x9 | 0xa => {
                let note = Note::new(first_byte);
                let velocity = self.read_byte();

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
                value: self.read_byte(),
            },
            0xc => PatchChange {
                program: first_byte,
            },
            0xd => ChannelAftertouch {
                channel: first_byte,
            },
            0xe => PitchWheelChange {
                value: ((first_byte as u16) << 7) | self.read_byte() as u16,
            },
            0xf => {
                if status.channel == 0xf {
                    let typ = first_byte;

                    if typ == 0x2f {
                        // End-of-track
                        let null_byte = self.read_byte();
                        assert!(null_byte == 0u8);
                        return None;
                    }

                    let length = self.reader.read_var_len().unwrap() as usize;
                    let data = self.read_bytes(length);

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
