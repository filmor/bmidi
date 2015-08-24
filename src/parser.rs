use std::io::{Read, BufReader};
use std::fs::File as FsFile;
use std::path::Path;

use errors::*;
use types::*;

use types::KeyEventType::*;
use types::EventType::*;

#[derive(Clone, Copy)]
struct Status {
    channel: Byte,
    opcode: Byte
}

struct MidiReader<'a> {
    reader: &'a mut Read,
    running_status: Status,
}

impl<'a> MidiReader<'a> {
    fn new(reader: &'a mut Read) -> MidiReader<'a> {
        MidiReader {
            reader: reader,
            running_status: Status { channel: 0, opcode: 0 },
        }
    }

    fn read_byte(&mut self) -> u8 {
        let mut res = [0 as u8];
        let len = self.reader.read(&mut res).unwrap();
        assert!(len == 1);
        res[0]
    }

    fn read_short(&mut self) -> u16 {
        let mut res = [0 as u8; 2];
        let len = self.reader.read(&mut res).unwrap();
        assert!(len == 2);
        (res[0] as u16) << 8 | (res[1] as u16)
    }

    fn read_int(&mut self) -> u32 {
        let mut res = [0 as u8; 4];
        let len = self.reader.read(&mut res).unwrap();
        assert!(len == 4);
        (((res[0] as u32) << 8
          | (res[1] as u32)) << 8
          | (res[2] as u32)) << 8
          | (res[3] as u32)
    }

    fn read_var_len(&mut self) -> u32 {
        let mut res = 0 as u32;

        loop {
            let next_byte = self.read_byte() as u32;
            res <<= 7;
            res |= next_byte & 0x7f;
            if next_byte & 0x80 == 0 { break }
        }

        res
    }

    fn read_bytes(&mut self, length: usize) -> Vec<u8> {
        let mut res = vec![0u8; length];
        self.reader.read(&mut res).unwrap();
        res
    }

    fn read_string(&mut self, length: usize) -> String {
        String::from_utf8(self.read_bytes(length)).unwrap()
    }
}

impl<'a> Iterator for MidiReader<'a> {
    type Item = Result<Event, MidiError>;

    fn next(&mut self) -> Option<Result<Event, MidiError>> {
        let ticks = self.read_var_len();

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

                    let length = self.read_var_len() as usize;
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

impl File {
    pub fn parse(filename: &Path) -> File {
        let f = FsFile::open(filename).unwrap();
        let mut reader = BufReader::new(f);
        let mut reader = MidiReader::new(&mut reader);

        let header = reader.read_string(4);

        if header != "MThd" {
            panic!("Not a MIDI file")
        }

        if reader.read_int() != 6 {
            panic!("Still not a MIDI file")
        }

        let format = reader.read_short();
        if format >= 3 {
            panic!("Version greater than 2, not implemented")
        }

        let track_count = reader.read_short();
        let mut tracks = Vec::<Track>::with_capacity(track_count as usize);
        let division = reader.read_short() as u32;

        println!("Found {} tracks, division {}", track_count, division);

        for _ in 0..track_count {
            tracks.push(File::parse_track(&mut reader))
        }

        for t in &tracks {
            println!("Parsed track of length {}", t.len());
        }

        File { format: format, division: division, tracks: tracks }
    }

    fn parse_track(reader: &mut MidiReader) -> Track {
        let header = reader.read_string(4);

        if header != "MTrk" {
            panic!("Invalid track header")
        }

        let length = reader.read_int() as usize;
        // TODO: Implement proper "Chunk" classes

        println!("Found track of length {}", length);

        reader.map(Result::unwrap)
              .collect()
    }
}
