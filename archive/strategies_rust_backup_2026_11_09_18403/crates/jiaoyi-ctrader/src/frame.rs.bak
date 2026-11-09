//! Framing of the cTrader Open API TCP stream.
//!
//! Each frame is a 4-byte big-endian length followed by that many bytes of a
//! serialised `ProtoMessage { payloadType, payload, clientMsgId }`.
//! Source: Spotware, "Protocol Buffers and JSON"
//! (help.ctrader.com/open-api/protocol-buffers-json/): a 4-byte "Message
//! Length" field, then the "Serialized ProtoMessage object". The page notes that
//! on a little-endian machine the length bytes must be reversed when sending
//! and receiving, i.e. the wire order is big-endian.

use prost::Message;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::pb::ProtoMessage;

/// Largest frame accepted from the server. The symbol list is the largest
/// response phase 1 reads; it is far below this.
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),
    #[error("frame of {0} bytes exceeds the {MAX_FRAME_BYTES}-byte limit")]
    TooLong(usize),
    #[error("frame is not a ProtoMessage: {0}")]
    Decode(#[from] prost::DecodeError),
}

/// Encode a `ProtoMessage` as one frame: length prefix plus message bytes.
pub fn encode_frame(msg: &ProtoMessage) -> Vec<u8> {
    let body = msg.encode_to_vec();
    let mut out = Vec::with_capacity(4 + body.len());
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(&body);
    out
}

/// Decode one complete frame from a byte slice. Returns the message and the
/// number of bytes consumed, or `None` if the slice holds less than a frame.
pub fn decode_frame(buf: &[u8]) -> Result<Option<(ProtoMessage, usize)>, FrameError> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(FrameError::TooLong(len));
    }
    if buf.len() < 4 + len {
        return Ok(None);
    }
    let msg = ProtoMessage::decode(&buf[4..4 + len])?;
    Ok(Some((msg, 4 + len)))
}

/// Read one frame from a stream.
pub async fn read_frame<R: AsyncRead + Unpin>(r: &mut R) -> Result<ProtoMessage, FrameError> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len).await?;
    let len = u32::from_be_bytes(len) as usize;
    if len > MAX_FRAME_BYTES {
        return Err(FrameError::TooLong(len));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body).await?;
    Ok(ProtoMessage::decode(body.as_slice())?)
}

/// Write one frame to a stream. This does NOT check the allowlist; the only
/// caller in the client, `Connection::send_message`, does that first.
pub(crate) async fn write_frame<W: AsyncWrite + Unpin>(
    w: &mut W,
    msg: &ProtoMessage,
) -> Result<(), FrameError> {
    w.write_all(&encode_frame(msg)).await?;
    w.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg() -> ProtoMessage {
        ProtoMessage {
            payload_type: 2137,
            payload: Some(vec![1, 2, 3, 250]),
            client_msg_id: Some("42".into()),
        }
    }

    #[test]
    fn encode_decode_round_trip() {
        let bytes = encode_frame(&msg());
        let (back, used) = decode_frame(&bytes).unwrap().unwrap();
        assert_eq!(back, msg());
        assert_eq!(used, bytes.len());
    }

    #[test]
    fn length_prefix_is_four_bytes_big_endian() {
        // ProtoMessage{payloadType=51} encodes as tag 0x08, varint 51 (0x33).
        let heartbeat = ProtoMessage {
            payload_type: 51,
            payload: None,
            client_msg_id: None,
        };
        assert_eq!(encode_frame(&heartbeat), vec![0, 0, 0, 2, 0x08, 0x33]);
    }

    #[test]
    fn partial_frame_waits_for_more_bytes() {
        let bytes = encode_frame(&msg());
        assert!(decode_frame(&bytes[..3]).unwrap().is_none());
        assert!(decode_frame(&bytes[..bytes.len() - 1]).unwrap().is_none());
    }

    #[test]
    fn oversized_length_is_refused() {
        let mut bytes = ((MAX_FRAME_BYTES + 1) as u32).to_be_bytes().to_vec();
        bytes.extend_from_slice(&[0; 8]);
        assert!(matches!(decode_frame(&bytes), Err(FrameError::TooLong(_))));
    }

    #[tokio::test]
    async fn async_read_write_round_trip_over_a_stream() {
        let (mut a, mut b) = tokio::io::duplex(1024);
        write_frame(&mut a, &msg()).await.unwrap();
        write_frame(&mut a, &msg()).await.unwrap();
        assert_eq!(read_frame(&mut b).await.unwrap(), msg());
        assert_eq!(read_frame(&mut b).await.unwrap(), msg());
    }
}
