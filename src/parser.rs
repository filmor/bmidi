use std::io::{Read, BufReader};
use std::fs::File as FsFile;
use std::path::Path;
use std::str;
use types::*;

use types::KeyEventType::*;
use types::EventType::*;

trait MidiReader {
    fn read_byte(&mut self) -> u8;
    fn read_short(&mut self) -> u16;
    fn read_int(&mut self) -> u32;
    fn read_var_len(&mut self) -> u32;
}

impl<T: Read> MidiReader for T {
    fn read_byte(&mut self) -> u8 {
        let mut res = [0 as u8];
        self.read(&mut res);
        res[0]
    }

    fn read_short(&mut self) -> u16 {
        let mut res = [0 as u8; 2];
        self.read(&mut res);
        (res[0] as u16) << 8 | (res[1] as u16)
    }

    fn read_int(&mut self) -> u32 {
        let mut res = [0 as u8; 4];
        self.read(&mut res);
        (((res[0] as u32) << 8 | (res[1] as u32)) << 8 | (res[2] as u32)) << 8 | (res[3] as u32)
    }

    fn read_var_len(&mut self) -> u32 {
        let mut res = 0 as u32;

        while {
            let next_byte = self.read_byte() as u32;
            res <<= 7;
            res |= (next_byte & 0x7f);
            next_byte & 0x80 != 0
        } {}

        res
    }
}

impl File {
    pub fn parse(filename: &Path) -> File {
        let mut f = FsFile::open(filename).unwrap();
        let mut reader = BufReader::new(f);

        let mut header = [0 as u8; 4];
        reader.read(&mut header);

        if str::from_utf8(&header).unwrap() != "MThd" {
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
        let tracks = Vec::<Track>::with_capacity(track_count as usize);
        let division = reader.read_short();

        File { format: format, division: 0, tracks: tracks }
    }

    fn parse_track(mut reader: &mut Read) -> Track {
        let mut header = [0 as u8; 4];
        reader.read(&mut header);

        if str::from_utf8(&header).unwrap() != "MTrk" {
            panic!("Invalid track header")
        }

        let length = reader.read_int() as usize;

        let mut res = Track::with_capacity(length);

        struct RunningStatus {
            channel: Byte,
            opcode: Byte
        }

        let mut status = RunningStatus { channel: 0, opcode: 0 };

        while true {
            let ticks = reader.read_var_len();

            let status_byte = reader.read_byte();

            status.channel = status_byte & 0x0f;
            status.opcode = (status_byte & 0xf0) >> 4;

            let event_type = match status.opcode {
                0x8 | 0x9 | 0xa => {
                    let note = reader.read_byte();
                    let velocity = reader.read_byte();

                    let typ = 
                        if status.opcode == 0x8 || (
                            status.opcode == 0xa && velocity == 0) {
                                Release
                        }
                        else if status.opcode == 0xa {
                            Press
                        }
                        else { Aftertouch };

                    Key {typ: typ, note: note, velocity: velocity}
                },
                0xb => ControlChange {
                    controller: reader.read_byte(),
                    value: reader.read_byte()
                },
                0xc => PatchChange {
                    program: reader.read_byte()
                },
                0xd => ChannelAftertouch {
                    channel: reader.read_byte()
                },
                0xe => PitchWheelChange {
                    value: ((reader.read_byte() as u16) << 7) | 
                        reader.read_byte() as u16
                },
                0xf => {
                    if status.channel == 0xf {
                        let typ = reader.read_byte();

                        if typ == 0x2f {
                            // End-of-track
                            reader.read_byte();
                            break;
                        }

                        let length = reader.read_var_len() as usize;
                        let mut data = Vec::<u8>::with_capacity(length);
                        unsafe { data.set_len(length) }
                        reader.read(data.as_mut_slice());

                        Meta { typ: typ, data: data }
                    }
                    else {
                        panic!("Nope")
                    }
                },
                _ => {
                    panic!("Unknown event")
                }
            };

            let event = Event {
                delay: ticks, channel: status.channel, typ: event_type
            };

            res.push(event);
        }

        res
    }
}
