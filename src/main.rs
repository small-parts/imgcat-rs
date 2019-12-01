use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{stdout, BufReader};
use std::process;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("Usage: imgcat <filename>");
        process::exit(1);
    }

    let filename = args.get(1).unwrap();
    let current_dir = env::current_dir().unwrap();
    let image = File::open(current_dir.join(filename)).unwrap();
    let mut image_reader = BufReader::new(image);

    let mut data = Vec::new();
    image_reader.read_to_end(&mut data).unwrap();

    concatenate_and_print_image(data).expect("display image failed");
}

fn concatenate_and_print_image(data: Vec<u8>) -> io::Result<()> {
    let mut image_writer = BufWriter::new(stdout());
    let is_tmux = env!("TERM").starts_with("screen");
    let mut buffer = Vec::<u8>::new();

    // OSC
    buffer.push(27);
    if is_tmux {
        buffer.extend(&[80, 116, 109, 117, 120, 59, 27, 27]);
    }
    buffer.push(b']');

    buffer.extend_from_slice(b"1337;File=");

    buffer.extend_from_slice(format!(";size={}", data.len()).as_bytes());
    buffer.extend_from_slice(b";inline=1");
    buffer.extend_from_slice(b";width=auto");
    buffer.extend_from_slice(b";height=auto");
    buffer.extend_from_slice(b";preserveAspectRatio=0");
    buffer.push(b':');

    buffer.extend_from_slice(base64::encode(&data).as_bytes());

    // ST
    buffer.push(7);
    if is_tmux {
        buffer.extend(&[27, 92]);
    }
    buffer.push(b'\n');

    image_writer.write_all(&buffer)?;
    image_writer.flush()
}
