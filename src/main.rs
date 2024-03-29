use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{stdout, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

fn main() {
    let config: Config = Config::from_args();
    concatenate_and_print_image(config).expect("failed display image");
}

#[derive(Debug, StructOpt)]
#[structopt(name = "imgcat")]
struct Config {
    #[structopt(parse(from_os_str))]
    path: PathBuf,

    #[structopt(long, default_value = "auto")]
    width: String,

    #[structopt(long, default_value = "auto")]
    height: String,

    #[structopt(long)]
    preserve_aspect_ratio: bool,

    #[structopt(long)]
    inline: bool,
}

fn concatenate_and_print_image(config: Config) -> io::Result<()> {
    let Config {
        path,
        width,
        height,
        preserve_aspect_ratio,
        inline,
    } = config;

    let data = read_file(&path)?;

    let is_tmux = env!("TERM").starts_with("screen");
    let mut buffer = Vec::<u8>::new();

    // OSC
    buffer.push(27);
    if is_tmux {
        buffer.extend(&[80, 116, 109, 117, 120, 59, 27, 27]);
    }
    buffer.push(b']');

    buffer.extend_from_slice(b"1337;File=");
    if let Some(filename) = path.file_name() {
        buffer.extend_from_slice(filename.as_bytes());
    }
    buffer.extend_from_slice(format!(";size={}", data.len()).as_bytes());
    buffer.extend_from_slice(format!(";inline={}", !inline as u8).as_bytes());
    buffer.extend_from_slice(format!(";width={}", width).as_bytes());
    buffer.extend_from_slice(format!(";height={}", height).as_bytes());
    buffer.extend_from_slice(format!(";preserveAspectRatio={}", preserve_aspect_ratio as u8).as_bytes());
    buffer.push(b':');

    buffer.extend_from_slice(base64::encode(&data).as_bytes());

    // ST
    buffer.push(7);
    if is_tmux {
        buffer.extend(&[27, 92]);
    }
    buffer.push(b'\n');

    let mut image_writer = BufWriter::new(stdout());
    image_writer.write_all(&buffer)?;
    image_writer.flush()
}

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}
