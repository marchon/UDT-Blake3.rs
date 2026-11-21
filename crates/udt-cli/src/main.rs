//! `udtcat` — stream files over pure-Rust UDT (Phase 5).
//!
//! Usage:
//! ```text
//! udtcat -l [host:]port [-o outfile]     # listen, write stream to file/stdout
//! udtcat host:port [-i infile]           # connect, send file/stdin
//! udtcat -h | --help
//! ```

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use udt_cli::{connect_and_send, listen_and_recv, open_input, open_output, parse_addr};

fn usage() -> ! {
    eprintln!(
        "\
udtcat — UDT stream file transfer (pure Rust)

Usage:
  udtcat -l [host:]port [-o outfile]   Listen; write received bytes to outfile (default stdout)
  udtcat host:port [-i infile]         Connect; send infile (default stdin)
  udtcat -h | --help                   Show this help

Examples:
  udtcat -l 9000 -o out.bin
  udtcat 127.0.0.1:9000 -i in.bin
"
    );
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }

    let listen = args.iter().position(|a| a == "-l").map(|i| {
        args.remove(i);
        true
    });

    let mut outfile: Option<PathBuf> = None;
    if let Some(i) = args.iter().position(|a| a == "-o") {
        args.remove(i);
        let p = args
            .get(i)
            .cloned()
            .unwrap_or_else(|| usage());
        args.remove(i);
        outfile = Some(PathBuf::from(p));
    }

    let mut infile: Option<PathBuf> = None;
    if let Some(i) = args.iter().position(|a| a == "-i") {
        args.remove(i);
        let p = args
            .get(i)
            .cloned()
            .unwrap_or_else(|| usage());
        args.remove(i);
        infile = Some(PathBuf::from(p));
    }

    let addr_s = args.first().cloned().unwrap_or_else(|| usage());
    let addr = match parse_addr(&addr_s) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("udtcat: {e}");
            return ExitCode::from(2);
        }
    };

    let result = if listen.unwrap_or(false) {
        let mut out = match open_output(outfile.as_deref()) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("udtcat: {e}");
                return ExitCode::FAILURE;
            }
        };
        listen_and_recv(addr, &mut *out).map(|n| {
            eprintln!("udtcat: received {n} bytes");
        })
    } else {
        let mut input = match open_input(infile.as_deref()) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("udtcat: {e}");
                return ExitCode::FAILURE;
            }
        };
        connect_and_send(addr, &mut *input).map(|n| {
            eprintln!("udtcat: sent {n} bytes");
        })
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("udtcat: {e}");
            ExitCode::FAILURE
        }
    }
}
