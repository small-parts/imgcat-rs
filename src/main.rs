use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::io::stdout;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use clap::Parser;

fn main() -> io::Result<()> {
    let args = Args::parse();

    let content = fs::read(&args.file_path)?;

    let is_tmux = env::var("TERM").is_ok_and(|term| term.starts_with("screen"));
    let mut buffer = Vec::new();

    // OSC
    buffer.push(b'\x1b');
    if is_tmux {
        buffer.extend_from_slice(b"Ptmux;\x1b\x1b");
    }
    buffer.push(b']');

    buffer.extend_from_slice(b"1337;File=");
    if let Some(filename) = args.file_path.file_name() {
        buffer.extend_from_slice(filename.as_bytes());
    }
    write!(buffer, ";size={}", content.len())?;
    buffer.extend_from_slice(b";inline=1");
    write!(buffer, ";width={}", args.width)?;
    write!(buffer, ";height={}", args.height)?;
    write!(buffer, ";preserveAspectRatio={}", u8::from(args.preserve_aspect_ratio))?;
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

#[derive(Parser)]
#[command(
    name = "imgcat",
    version,
    about = "Display images inline in iTerm2",
    long_about = "Read an image file and print the iTerm2 inline image escape sequence to stdout."
)]
struct Args {
    /// Image file to display.
    file_path: PathBuf,

    /// Display width.
    #[arg(long, default_value = "auto")]
    width: String,

    /// Display height.
    #[arg(long, default_value = "auto")]
    height: String,

    /// Preserve the image aspect ratio.
    #[arg(long)]
    preserve_aspect_ratio: bool,
}
