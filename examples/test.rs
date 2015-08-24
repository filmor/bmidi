extern crate midi;

use midi::*;

fn main() {
    let res = File::parse("test.mid".as_ref());

    for t in res.tracks.iter() {
        println!("\nTrack:\n=========");
        for ev in t.iter() {
            println!("{:?}", ev);
        }
    }
}
