use std::env;
use std::fs;
use std::io::{self, Write, stdout};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use clap::Arg;
use clap::ArgAction;
use clap::Command;
use clap::value_parser;

fn main() -> io::Result<()> {
    let matches = Command::new("imgcat")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Display images inline in iTerm2")
        .long_about("Read an image file and print the iTerm2 inline image escape sequence to stdout.")
        .arg(
            Arg::new("file_path")
                .help("Image file to display.")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .help("Display width.")
                .default_value("auto"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .help("Display height.")
                .default_value("auto"),
        )
        .arg(
            Arg::new("preserve_aspect_ratio")
                .long("preserve-aspect-ratio")
                .help("Preserve the image aspect ratio.")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    let file_path = matches
        .get_one::<PathBuf>("file_path")
        .expect("`file_path` is required");
    let width = matches.get_one::<String>("width").expect("`width` has a default value");
    let height = matches
        .get_one::<String>("height")
        .expect("`height` has a default value");
    let preserve_aspect_ratio = matches.get_flag("preserve_aspect_ratio");

    let content = fs::read(file_path)?;

    let is_tmux = env::var("TERM").is_ok_and(|term| term.starts_with("screen"));
    let mut buffer = Vec::new();

    // OSC
    buffer.push(b'\x1b');
    if is_tmux {
        buffer.extend_from_slice(b"Ptmux;\x1b\x1b");
    }
    buffer.push(b']');

    buffer.extend_from_slice(b"1337;File=");
    if let Some(filename) = file_path.file_name() {
        buffer.extend_from_slice(filename.as_bytes());
    }
    write!(buffer, ";size={}", content.len())?;
    buffer.extend_from_slice(b";inline=1");
    write!(buffer, ";width={}", width)?;
    write!(buffer, ";height={}", height)?;
    write!(buffer, ";preserveAspectRatio={}", u8::from(preserve_aspect_ratio))?;
    buffer.push(b':');

    buffer.extend_from_slice(BASE64_STANDARD.encode(&content).as_bytes());

    // ST
    buffer.push(b'\x07');
    if is_tmux {
        buffer.extend_from_slice(b"\x1b\\");
    }
    buffer.push(b'\n');

    let mut stdout = stdout().lock();
    stdout.write_all(&buffer)?;
    stdout.flush()
}
