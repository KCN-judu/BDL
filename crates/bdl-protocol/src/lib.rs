//! Studio ↔ bdld protocol.
//!
//! * [`pb`]      — prost-generated message types from `proto/bdl/v1/bdl.proto`.
//! * [`framing`] — length-prefixed frame codec; transport-agnostic.
//! * [`convert`] — conversions between protocol messages and the project
//!   model (`bdl-model`).  Only Rust performs these; Dart sees protobuf only.
//! * [`PROTOCOL_VERSION`] — bumped on any breaking wire change.

#![forbid(unsafe_code)]

pub mod convert;
pub mod framing;

#[allow(clippy::all, missing_docs)]
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/bdl.v1.rs"));
}

/// Semantic version of the wire protocol.  Clients and daemons with equal
/// `major` are compatible.
pub const PROTOCOL_VERSION: pb::Version = pb::Version {
    major: 0,
    minor: 9,
    patch: 0,
};

pub fn compatible(client: &pb::Version, daemon: &pb::Version) -> bool {
    client.major == daemon.major
}
