extern crate midi;

use midi::File;

fn main() {
    let res = File::parse("test.mid".as_ref());
}
