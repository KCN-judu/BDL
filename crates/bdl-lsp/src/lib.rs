//! `bdl-lsp`: the Language Server Protocol adapter over `bdl-ide`.
//!
//! LSP is an adapter, not the language service: this crate owns JSON-RPC
//! transport, the byte-offset ↔ position conversion for the negotiated
//! encoding ([`position`]), and the rendering of `bdl-ide` results as LSP
//! types ([`convert`]).  It holds no semantic logic — no parsing, typing,
//! name resolution or reference search — and no unsaved-buffer store of
//! its own: `didOpen`/`didChange`/`didClose` are overlay updates on the
//! same [`bdl_ide::IdeHost`] Studio's drafts go through.
//!
//! Stack: `lsp-server` (synchronous message loop, any transport, no
//! runtime coupling) + `lsp-types` 0.97 (LSP 3.17, pull diagnostics).
//! See ADR-0016.

#![forbid(unsafe_code)]

pub mod convert;
pub mod position;
pub mod server;

pub use position::{LineIndex, PositionEncoding};
pub use server::{
    run, ExplainEntity, InvalidationPreviewRequest, PreviewEdit, VirtualDocumentParams,
    VirtualDocumentRequest, VirtualDocumentResult,
};
