//! Frame = 4-byte big-endian length prefix + protobuf payload.
//!
//! Pure functions over byte buffers; the async transport in `bdl-daemon`
//! and the Dart client both implement exactly this.

use bytes::{Buf, BufMut, BytesMut};
use prost::Message;

/// Hard upper bound on one frame, to reject corrupt length prefixes early.
pub const MAX_FRAME_LEN: usize = 64 * 1024 * 1024;
pub const HEADER_LEN: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame of {len} bytes exceeds the {MAX_FRAME_LEN}-byte limit")]
    TooLarge { len: usize },
    #[error("protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("protobuf encode error: {0}")]
    Encode(#[from] prost::EncodeError),
}

/// Append `msg` as one frame to `out`.
pub fn encode<M: Message>(msg: &M, out: &mut BytesMut) -> Result<(), FrameError> {
    let len = msg.encoded_len();
    if len > MAX_FRAME_LEN {
        return Err(FrameError::TooLarge { len });
    }
    out.reserve(HEADER_LEN + len);
    out.put_u32(len as u32);
    Ok(msg.encode(out)?)
}

/// Try to take one complete frame from the front of `buf`.
///
/// Returns `Ok(None)` when more bytes are needed (buffer untouched).
pub fn decode<M: Message + Default>(buf: &mut BytesMut) -> Result<Option<M>, FrameError> {
    if buf.len() < HEADER_LEN {
        return Ok(None);
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if len > MAX_FRAME_LEN {
        return Err(FrameError::TooLarge { len });
    }
    if buf.len() < HEADER_LEN + len {
        return Ok(None);
    }
    buf.advance(HEADER_LEN);
    let payload = buf.split_to(len);
    Ok(Some(M::decode(payload.freeze())?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb;

    #[test]
    fn round_trip_and_partial_reads() {
        let msg = pb::ClientMessage {
            request_id: 7,
            payload: Some(pb::client_message::Payload::Handshake(
                pb::HandshakeRequest {
                    client_protocol_version: Some(crate::PROTOCOL_VERSION),
                    client_name: "test".into(),
                    client_version: "0".into(),
                },
            )),
        };
        let mut buf = BytesMut::new();
        encode(&msg, &mut buf).unwrap();
        encode(&msg, &mut buf).unwrap();
        let whole = buf.clone();

        // feed one byte at a time; nothing decodes until the frame is complete
        let mut partial = BytesMut::new();
        let mut decoded = Vec::new();
        for b in whole.iter() {
            partial.put_u8(*b);
            while let Some(m) = decode::<pb::ClientMessage>(&mut partial).unwrap() {
                decoded.push(m);
            }
        }
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0], msg);
        assert!(partial.is_empty());
    }

    #[test]
    fn oversized_prefix_is_rejected() {
        let mut buf = BytesMut::new();
        buf.put_u32(u32::MAX);
        assert!(matches!(
            decode::<pb::ClientMessage>(&mut buf),
            Err(FrameError::TooLarge { .. })
        ));
    }
}
