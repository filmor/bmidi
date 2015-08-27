use std::io::Read;

use types::*;
use types::KeyEventType::*;
use types::EventType::*;

use errors::*;

#[derive(Clone, Copy)]
struct Status {
    channel: Byte,
    opcode: Byte
}

pub struct MidiReader<'a> {
    reader: Box<MidiRead + 'a>,
    running_status: Status,
}

pub trait MidiRead {
    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError>;

    fn read_byte(&mut self) -> Result<u8, MidiError> {
        let mut res = [0 as u8];
        try!(self.read(&mut res));
        Ok(res[0])
    }

    fn read_short(&mut self) -> Result<u16, MidiError> {
        let mut res = [0 as u8; 2];
        try!(self.read(&mut res));
        Ok((res[0] as u16) << 8 | (res[1] as u16))
    }

    fn read_int(&mut self) -> Result<u32, MidiError> {
        let mut res = [0 as u8; 4];
        try!(self.read(&mut res));
        Ok((((res[0] as u32) << 8
          | (res[1] as u32)) << 8
          | (res[2] as u32)) << 8
          | (res[3] as u32))
    }

    fn read_var_len(&mut self) -> Result<u32, MidiError> {
        let mut res = 0 as u32;

        loop {
            let next_byte = try!(self.read_byte()) as u32;
            res <<= 7;
            res |= next_byte & 0x7f;
            if next_byte & 0x80 == 0 { break }
        }

        Ok(res)
    }
}

/* impl<T: Iterator<Item=u8>> MidiRead for T {
    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError> {
        for field in output.iter_mut() {
            match self.next() {
                Some(value) => *field = value,
                None => return Err(MidiError::EndOfStream)
            }
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, MidiError> {
        match self.next() {
            Some(value) => Ok(value),
            None => Err(MidiError::EndOfStream)
        }
    }
}*/

impl<T: Read> MidiRead for T {
    fn read(&mut self, output: &mut [u8]) -> Result<(), MidiError> {
        match Read::read(self, output).ok() {
            Some(len) if len == output.len() => Ok(()),
            _ => Err(MidiError::EndOfStream)
        }
    }
}

impl<'a> MidiReader<'a> {
    pub fn new<T: Read>(reader: &'a mut T) -> MidiReader<'a> {
        MidiReader {
            reader: Box::new(reader) as Box<MidiRead + 'a>,
            running_status: Status { channel: 0, opcode: 0 },
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

    fn read_bytes(&mut self, length: usize) -> Vec<u8> {
        let mut res = vec![0u8; length];
        self.reader.read(&mut res).unwrap();
        res
    }

    pub fn read_string(&mut self, length: usize) -> String {
        String::from_utf8(self.read_bytes(length)).unwrap()
    }
}

impl<'a> Iterator for MidiReader<'a> {
    type Item = Result<Event, MidiError>;

    fn next(&mut self) -> Option<Result<Event, MidiError>> {
        let ticks = self.reader.read_var_len().unwrap();

        let mut first_byte = self.read_byte();

        if (first_byte & 1 << 7) != 0 {
            let status_byte = first_byte;
            first_byte = self.read_byte();
            self.running_status = Status {
                channel: status_byte & 0xf,
                opcode: (status_byte & 0xf0) >> 4
            };
        } else {
            self.running_status;
        }

        let status = self.running_status;

        let event_type = match status.opcode {
            0x8 | 0x9 | 0xa => {
                let note = Note::new(first_byte);
                let velocity = self.read_byte();

                let typ = 
                    if status.opcode == 0x8 || (
                        status.opcode == 0x9 && velocity == 0) {
                            Release
                    }
                    else if status.opcode == 0x9 {
                        Press
                    }
                    else { Aftertouch };

                Key {typ: typ, note: note, velocity: velocity}
            },
            0xb => ControlChange {
                controller: first_byte,
                value: self.read_byte()
            },
            0xc => PatchChange {
                program: first_byte
            },
            0xd => ChannelAftertouch {
                channel: first_byte
            },
            0xe => PitchWheelChange {
                value: ((first_byte as u16) << 7) | self.read_byte() as u16
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

                    Meta { typ: typ, data: data }
                }
                else {
                    panic!("Nope")
                }
            },
            _ => unreachable!("Invalid opcode")
        };

        let event = Event {
            delay: ticks, channel: status.channel, typ: event_type
        };

        Some(Ok(event))
    }
}
