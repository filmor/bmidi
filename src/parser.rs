use std::io::{Read, BufReader};
use std::fs::File as FsFile;
use std::path::Path;

use types::{Event, Ticks};
use reader::MidiReader;
use combined_iterator::CombinedIterator;

pub type Track = Vec<u8>;

pub struct File {
    pub tracks: Vec<Track>,
    pub format: u16,
    pub division: Ticks,
}

impl File {
    // TODO: Use explicit lifetime 
    pub fn track_iter<'a>(&'a self, index: usize) -> Box<Iterator<Item=Event> + 'a> {
        let ref track = self.tracks[index];
        let iter = track.into_iter().map(|x| *x);
        let my_reader = MidiReader::new(iter);
        Box::new(my_reader)
    }

    pub fn iter<'a>(&'a self) -> CombinedIterator<'a> {
        CombinedIterator::<'a>::new(
            (0..self.tracks.len()).map(|n| self.track_iter(n)).collect()
            )
    }

    pub fn parse(filename: &Path) -> File {
        let f = FsFile::open(filename).unwrap();
        let reader = BufReader::new(f);
        let mut reader = reader.bytes().map(Result::unwrap);
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
            let header = reader.read_string(4);

            if header != "MTrk" {
                panic!("Invalid track header")
            }

            let length = reader.read_int() as usize;

            println!("Found track of length {}", length);

            tracks.push(reader.read_bytes(length))
        }

        for t in &tracks {
            println!("Parsed track of length {}", t.len());
        }

        File { format: format, division: division, tracks: tracks }
    }
}
