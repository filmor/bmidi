extern crate midi;

use midi::*;
use std::thread;

fn main() {
    let res = File::parse("test.mid".as_ref());

    let tracks = res.tracks.clone();

    let mut handles = Vec::with_capacity(tracks.len());

    for t in tracks.iter() {
        let t = t.clone();
        let h = thread::spawn(move || {
            for ev in t.iter() {
                thread::sleep_ms(ev.delay);
                println!("{:?}", ev);
            }
        });
        handles.push(h);
    }

    for h in handles.into_iter() {
        h.join().unwrap();
    }

    // handles.iter().map(move |h| h.join()).collect::<Vec<_>>();
}
