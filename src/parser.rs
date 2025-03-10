use std::fs::File as FsFile;
use std::io::BufReader;
use std::path::Path;

use crate::combined_iterator::CombinedIterator;
use crate::errors::MidiError;
use crate::reader::MidiRead;
use crate::types::{Event, Ticks};

pub type Track = Vec<u8>;

pub struct File {
    pub tracks: Vec<Track>,
    pub format: u16,
    pub division: Ticks,
}

impl File {
    pub fn track_iter<'a>(&'a self, index: usize) -> Box<dyn Iterator<Item = Event> + 'a> {
        let track = &self.tracks[index];
        let my_reader = MidiRead::new(std::io::Cursor::new(track));
        Box::new(my_reader)
    }

    pub fn iter<'a>(&'a self) -> CombinedIterator<'a> {
        CombinedIterator::<'a>::new((0..self.tracks.len()).map(|n| self.track_iter(n)).collect())
    }

    pub fn parse(filename: &Path) -> Result<File, MidiError> {
        let f = FsFile::open(filename).unwrap();
        let reader = BufReader::new(f);
        let mut reader = MidiRead::new(reader);

        let header = reader.read_string(4)?;

        if header != "MThd" {
            return Err(MidiError::InvalidHeader);
        }

        if reader.read_int()? != 6 {
            return Err(MidiError::InvalidHeader);
        }

        let format = reader.read_short()?;
        if format >= 3 {
            return Err(MidiError::UnsupportedVersion);
        }

        let track_count = reader.read_short()?;
        let mut tracks = Vec::<Track>::with_capacity(track_count as usize);
        let division = reader.read_short()?.into();

        // println!("Found {} tracks, division {}", track_count, division);

        for _ in 0..track_count {
            let header = reader.read_string(4)?;

            if header != "MTrk" {
                panic!("Invalid track header")
            }

            let length = reader.read_int()? as usize;

            println!("Found track of length {}", length);

            tracks.push(reader.read_bytes_dyn(length)?)
        }

        for t in &tracks {
            println!("Parsed track of length {}", t.len());
        }

        Ok(File {
            format,
            division,
            tracks,
        })
    }
}
