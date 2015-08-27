use std::io::BufReader;
use std::fs::File as FsFile;
use std::path::Path;

use types::{Event, Ticks};
use reader::MidiReader;

pub type Track = Vec<Event>;

pub struct File {
    pub tracks: Vec<Track>,
    pub format: u16,
    pub division: Ticks,
}

impl File {
    pub fn parse(filename: &Path) -> File {
        let f = FsFile::open(filename).unwrap();
        let mut reader = BufReader::new(f);
        // let mut reader = reader.bytes().map(Result::unwrap);
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
