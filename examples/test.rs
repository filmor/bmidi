extern crate bmidi;

use bmidi::*;

fn main() {
    let res = File::parse("test.mid".as_ref());

    for t in 0..res.tracks.len() {
        println!("\nTrack:\n=========");
        for ev in res.track_iter(t) {
            println!("{:?}", ev);
        }
    }
}
