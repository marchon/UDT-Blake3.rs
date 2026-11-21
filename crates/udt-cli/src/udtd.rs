//! `udtd` — persistent UDT file-delivery daemon and client.
//!
//! ```text
//! udtd serve -l [host:]port -d inbox_dir
//! udtd put host:port file [file...]
//! ```

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use udt_cli::{parse_addr, put_files, serve_inbox};

fn usage() -> ! {
    eprintln!(
        "\
udtd — UDT multi-file delivery server / client

Usage:
  udtd serve -l [host:]port -d inbox_dir   Listen forever; save framed files under inbox_dir
  udtd put host:port file [file...]        Connect once per file; deliver framed contents
  udtd -h | --help                         Show this help

Frame: u32 name_len | name | u64 file_len | bytes  (big-endian)

Examples:
  udtd serve -l 19000 -d /var/udt/inbox
  udtd put 70.88.205.138:19000 ./report.pdf ./data.bin
"
    );
    std::process::exit(2);
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        usage();
    }

    let cmd = args.remove(0);
    match cmd.as_str() {
        "serve" => cmd_serve(&mut args),
        "put" => cmd_put(&mut args),
        _ => {
            eprintln!("udtd: unknown command '{cmd}'");
            usage();
        }
    }
}

fn cmd_serve(args: &mut Vec<String>) -> ExitCode {
    let mut listen: Option<String> = None;
    let mut inbox: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-l" => {
                i += 1;
                listen = args.get(i).cloned();
                i += 1;
            }
            "-d" => {
                i += 1;
                inbox = args.get(i).map(PathBuf::from);
                i += 1;
            }
            other => {
                eprintln!("udtd serve: unexpected argument '{other}'");
                usage();
            }
        }
    }
    let listen = listen.unwrap_or_else(|| usage());
    let inbox = inbox.unwrap_or_else(|| usage());
    let addr = match parse_addr(&listen) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("udtd: {e}");
            return ExitCode::from(2);
        }
    };
    // serve_inbox loops forever; Err is unexpected.
    match serve_inbox(addr, &inbox) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("udtd: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_put(args: &mut Vec<String>) -> ExitCode {
    if args.len() < 2 {
        usage();
    }
    let addr_s = args.remove(0);
    let addr = match parse_addr(&addr_s) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("udtd: {e}");
            return ExitCode::from(2);
        }
    };
    let paths: Vec<PathBuf> = args.iter().map(PathBuf::from).collect();
    for p in &paths {
        if !p.is_file() {
            eprintln!("udtd: not a file: {}", p.display());
            return ExitCode::from(2);
        }
    }
    match put_files(addr, &paths) {
        Ok(n) => {
            eprintln!("udtd: sent {n} bytes across {} file(s)", paths.len());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("udtd: {e}");
            ExitCode::FAILURE
        }
    }
}
