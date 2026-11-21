//! Print the planned PureRust ↔ CppUdt4 interop matrix.
//!
//! ```sh
//! cargo run -p udt-interop --example print_matrix
//! ```

use udt_interop::{planned_matrix, PeerKind};

fn peer_label(p: PeerKind) -> &'static str {
    match p {
        PeerKind::PureRust => "PureRust",
        PeerKind::CppUdt4 => "CppUdt4",
    }
}

fn main() {
    println!("planned interop matrix ({} cells):", planned_matrix().len());
    for (i, cell) in planned_matrix().iter().enumerate() {
        println!(
            "  [{i}] client={} server={}",
            peer_label(cell.client),
            peer_label(cell.server)
        );
    }
}
