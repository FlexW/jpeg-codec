mod jpeg;

use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = File::open("samples/lenna.jpg").expect("Failed to open file!");
    let mut decoder = jpeg::decoder::Decoder::new(BufReader::new(file));
    decoder.decode().expect("Could not decode");
}
