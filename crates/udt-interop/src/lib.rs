//! C++ UDT4 interop helpers (Phase 6).
//!
//! Default features do **not** link `libudt`. Enable `cpp-udt4` after the
//! vendored submodule at `vendor/udt` is pinned (ADR-0001).

#![deny(missing_docs)]

/// Describes one cell in the interop matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteropCell {
    /// Who initiates the connection.
    pub client: PeerKind,
    /// Who accepts.
    pub server: PeerKind,
}

/// Implementation under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerKind {
    /// This pure-Rust stack.
    PureRust,
    /// Reference C++ UDT4.
    CppUdt4,
}

/// Planned matrix (both directions).
pub fn planned_matrix() -> [InteropCell; 4] {
    use PeerKind::*;
    [
        InteropCell {
            client: PureRust,
            server: PureRust,
        },
        InteropCell {
            client: PureRust,
            server: CppUdt4,
        },
        InteropCell {
            client: CppUdt4,
            server: PureRust,
        },
        InteropCell {
            client: CppUdt4,
            server: CppUdt4,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_has_cross_impl_cells() {
        let m = planned_matrix();
        assert!(m.iter().any(|c| {
            matches!(
                (c.client, c.server),
                (PeerKind::PureRust, PeerKind::CppUdt4)
                    | (PeerKind::CppUdt4, PeerKind::PureRust)
            )
        }));
    }
}

/// Human label for a matrix cell (client→server).
pub fn cell_label(cell: InteropCell) -> String {
    format!("{:?}→{:?}", cell.client, cell.server)
}

/// Cells that require the vendored C++ peers.
pub fn cpp_cross_cells() -> [InteropCell; 2] {
    use PeerKind::*;
    [
        InteropCell {
            client: PureRust,
            server: CppUdt4,
        },
        InteropCell {
            client: CppUdt4,
            server: PureRust,
        },
    ]
}

#[cfg(test)]
mod phase6_tests {
    use super::*;

    #[test]
    fn cpp_cross_cells_are_planned() {
        let planned = planned_matrix();
        for cell in cpp_cross_cells() {
            assert!(planned.contains(&cell));
            assert!(!cell_label(cell).is_empty());
        }
    }
}

