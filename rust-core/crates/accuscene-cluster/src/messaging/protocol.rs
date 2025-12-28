//! Binary messaging protocol.

use crate::error::{ClusterError, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Protocol version.
pub const PROTOCOL_VERSION: u8 = 1;

/// Message type codes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Ping = 0x01,
    Pong = 0x02,
    Request = 0x10,
    Response = 0x11,
    Error = 0x12,
    Gossip = 0x20,
    Consensus = 0x30,
    Replication = 0x40,
}

impl MessageType {
    /// Convert from u8.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(MessageType::Ping),
            0x02 => Some(MessageType::Pong),
            0x10 => Some(MessageType::Request),
            0x11 => Some(MessageType::Response),
            0x12 => Some(MessageType::Error),
            0x20 => Some(MessageType::Gossip),
            0x30 => Some(MessageType::Consensus),
            0x40 => Some(MessageType::Replication),
            _ => None,
        }
    }

    /// Convert to u8.
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Message header.
#[derive(Debug, Clone)]
pub struct MessageHeader {
    /// Protocol version
    pub version: u8,

    /// Message type
    pub message_type: MessageType,

    /// Message ID
    pub message_id: Uuid,

    /// Source node ID
    pub source: Uuid,

    /// Destination node ID (optional)
    pub destination: Option<Uuid>,

    /// Payload length
    pub payload_length: u32,

    /// Checksum
    pub checksum: u32,
}

impl MessageHeader {
    /// Create a new message header.
    pub fn new(message_type: MessageType, source: Uuid) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            message_type,
            message_id: Uuid::new_v4(),
            source,
            destination: None,
            payload_length: 0,
            checksum: 0,
        }
    }

    /// Set destination.
    pub fn with_destination(mut self, destination: Uuid) -> Self {
        self.destination = Some(destination);
        self
    }

    /// Encode header to bytes.
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(256);

        // Version (1 byte)
        buf.put_u8(self.version);

        // Message type (1 byte)
        buf.put_u8(self.message_type.to_u8());

        // Message ID (16 bytes)
        buf.put_slice(self.message_id.as_bytes());

        // Source (16 bytes)
        buf.put_slice(self.source.as_bytes());

        // Destination (1 byte flag + optional 16 bytes)
        if let Some(dest) = self.destination {
            buf.put_u8(1);
            buf.put_slice(dest.as_bytes());
        } else {
            buf.put_u8(0);
        }

        // Payload length (4 bytes)
        buf.put_u32(self.payload_length);

        // Checksum (4 bytes)
        buf.put_u32(self.checksum);

        buf
    }

    /// Decode header from bytes.
    pub fn decode(buf: &mut impl Buf) -> Result<Self> {
        if buf.remaining() < 42 {
            return Err(ClusterError::InvalidMessage(
                "Insufficient header data".to_string(),
            ));
        }

        // Version
        let version = buf.get_u8();
        if version != PROTOCOL_VERSION {
            return Err(ClusterError::InvalidMessage(format!(
                "Unsupported protocol version: {}",
                version
            )));
        }

        // Message type
        let message_type_byte = buf.get_u8();
        let message_type = MessageType::from_u8(message_type_byte).ok_or_else(|| {
            ClusterError::InvalidMessage(format!("Invalid message type: {}", message_type_byte))
        })?;

        // Message ID
        let mut message_id_bytes = [0u8; 16];
        buf.copy_to_slice(&mut message_id_bytes);
        let message_id = Uuid::from_bytes(message_id_bytes);

        // Source
        let mut source_bytes = [0u8; 16];
        buf.copy_to_slice(&mut source_bytes);
        let source = Uuid::from_bytes(source_bytes);

        // Destination
        let has_destination = buf.get_u8() == 1;
        let destination = if has_destination {
            let mut dest_bytes = [0u8; 16];
            buf.copy_to_slice(&mut dest_bytes);
            Some(Uuid::from_bytes(dest_bytes))
        } else {
            None
        };

        // Payload length
        let payload_length = buf.get_u32();

        // Checksum
        let checksum = buf.get_u32();

        Ok(Self {
            version,
            message_type,
            message_id,
            source,
            destination,
            payload_length,
            checksum,
        })
    }

    /// Get header size.
    pub fn size(&self) -> usize {
        42 + if self.destination.is_some() { 16 } else { 0 }
    }
}

/// Complete message with header and payload.
#[derive(Debug, Clone)]
pub struct Message {
    /// Message header
    pub header: MessageHeader,

    /// Message payload
    pub payload: Bytes,
}

impl Message {
    /// Create a new message.
    pub fn new<T: Serialize>(
        message_type: MessageType,
        source: Uuid,
        payload: &T,
    ) -> Result<Self> {
        let payload_bytes = bincode::serialize(payload)?;
        let checksum = crc32fast::hash(&payload_bytes);

        let mut header = MessageHeader::new(message_type, source);
        header.payload_length = payload_bytes.len() as u32;
        header.checksum = checksum;

        Ok(Self {
            header,
            payload: Bytes::from(payload_bytes),
        })
    }

    /// Decode payload as type T.
    pub fn decode_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        // Verify checksum
        let calculated_checksum = crc32fast::hash(&self.payload);
        if calculated_checksum != self.header.checksum {
            return Err(ClusterError::ChecksumMismatch);
        }

        bincode::deserialize(&self.payload).map_err(Into::into)
    }

    /// Encode message to bytes.
    pub fn encode(&self) -> BytesMut {
        let mut buf = self.header.encode();
        buf.put_slice(&self.payload);
        buf
    }

    /// Decode message from bytes.
    pub fn decode(mut buf: impl Buf) -> Result<Self> {
        let header = MessageHeader::decode(&mut buf)?;

        if buf.remaining() < header.payload_length as usize {
            return Err(ClusterError::InvalidMessage(
                "Insufficient payload data".to_string(),
            ));
        }

        let payload = buf.copy_to_bytes(header.payload_length as usize);

        Ok(Self { header, payload })
    }

    /// Get total message size.
    pub fn size(&self) -> usize {
        self.header.size() + self.payload.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        value: String,
    }

    #[test]
    fn test_message_header() {
        let source = Uuid::new_v4();
        let header = MessageHeader::new(MessageType::Ping, source);

        let encoded = header.encode();
        let mut buf = &encoded[..];
        let decoded = MessageHeader::decode(&mut buf).unwrap();

        assert_eq!(decoded.version, PROTOCOL_VERSION);
        assert_eq!(decoded.message_type, MessageType::Ping);
        assert_eq!(decoded.source, source);
    }

    #[test]
    fn test_message() {
        let source = Uuid::new_v4();
        let payload = TestPayload {
            value: "test".to_string(),
        };

        let message = Message::new(MessageType::Request, source, &payload).unwrap();
        let encoded = message.encode();

        let decoded = Message::decode(&encoded[..]).unwrap();
        let decoded_payload: TestPayload = decoded.decode_payload().unwrap();

        assert_eq!(decoded_payload, payload);
        assert_eq!(decoded.header.message_type, MessageType::Request);
    }
}
