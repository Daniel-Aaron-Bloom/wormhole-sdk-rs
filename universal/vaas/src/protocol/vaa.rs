use wormhole_io::deploys::ChainId;

use crate::{
    payloads::{self, PayloadKind},
    utils, TypePrefixedPayload,
};
pub use crate::{GuardianSetSig, Readable, Writeable};

use std::{
    fmt, io,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vaa {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub header: VaaHeader,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub body: VaaBody,
}

impl Readable for Vaa {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        let header = VaaHeader::read(reader)?;
        let body = VaaBody::read(reader)?;
        Ok(Self { header, body })
    }
}

impl Writeable for Vaa {
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.header.write(writer)?;
        self.body.write(writer)?;
        Ok(())
    }

    fn written_size(&self) -> usize {
        self.header.written_size() + self.body.written_size()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VaaHeader {
    pub version: u8,
    pub guardian_set_index: u32,
    pub signatures: Vec<GuardianSetSig>,
}

impl Writeable for VaaHeader {
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(&[self.version])?;
        writer.write_all(&self.guardian_set_index.to_be_bytes())?;
        writer.write_all(&[self.signatures.len() as u8])?;
        self.signatures
            .iter()
            .try_for_each(|sig| sig.write(writer))?;
        Ok(())
    }

    fn written_size(&self) -> usize {
        1 + 4 + (self.signatures.len() * <GuardianSetSig as Readable>::SIZE.unwrap())
    }
}

impl Readable for VaaHeader {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut buf = [0u8; 1 + 4 + 1];
        reader.read_exact(&mut buf)?;

        let version = buf[0];
        let guardian_set_index = u32::from_be_bytes(buf[1..5].try_into().unwrap());
        let sig_count = buf[5] as usize;

        let mut signatures: Vec<_> = Vec::with_capacity(sig_count);
        for _ in 0..sig_count {
            signatures.push(GuardianSetSig::read(reader)?);
        }

        Ok(Self {
            version,
            guardian_set_index,
            signatures,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VaaBody {
    pub timestamp: u32,
    pub nonce: u32,
    #[cfg_attr(
        feature = "serde",
        serde(serialize_with = "wormhole_io::deploys::chain_id::serde::serialize_value")
    )]
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "wormhole_io::deploys::chain_id::serde::deserialize_value")
    )]
    pub emitter_chain: ChainId,
    pub emitter_address: [u8; 32],
    pub sequence: u64,
    pub consistency_level: u8,

    pub payload: PayloadKind,
}

impl Writeable for VaaBody {
    fn written_size(&self) -> usize {
        self.timestamp.written_size()
            + self.nonce.written_size()
            + self.emitter_chain.written_size()
            + self.emitter_address.written_size()
            + self.sequence.written_size()
            + self.consistency_level.written_size()
            + self.payload.payload_written_size()
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.timestamp.write(writer)?;
        self.nonce.write(writer)?;
        self.emitter_chain.write(writer)?;
        self.emitter_address.write(writer)?;
        self.sequence.write(writer)?;
        self.consistency_level.write(writer)?;
        self.payload.write(writer)?;
        Ok(())
    }
}

impl Readable for VaaBody {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        Ok(Self {
            timestamp: Readable::read(reader)?,
            nonce: Readable::read(reader)?,
            emitter_chain: Readable::read(reader)?,
            emitter_address: Readable::read(reader)?,
            sequence: Readable::read(reader)?,
            consistency_level: Readable::read(reader)?,
            payload: Readable::read(reader)?,
        })
    }
}

impl VaaBody {
    #[allow(unreachable_patterns)]
    pub fn payload_bytes(&self) -> Option<&[u8]> {
        match &self.payload {
            PayloadKind::Binary(buf) => Some(buf),
            _ => None,
        }
    }

    pub fn read_payload<P: TypePrefixedPayload>(&self) -> Option<P> {
        let mut p = self.payload_bytes()?;
        let deser = P::read_payload(&mut p).ok()?;

        // Check that the payload is fully consumed. No extra bytes allowed.
        p.is_empty().then_some(deser)
    }

    pub fn payload_as_message(&self) -> Option<payloads::Message> {
        self.read_payload()
    }

    #[inline]
    pub fn digest(&self) -> MessageHash {
        MessageHash(utils::keccak256(self.to_vec()))
    }

    #[inline]
    pub fn double_digest(&self) -> VaaHash {
        VaaHash(utils::keccak256(self.digest()))
    }

    #[cfg(feature = "serde")]
    pub fn deser_payload<P: TypePrefixedPayload + serde::de::DeserializeOwned>(&self) -> Option<P> {
        match &self.payload {
            PayloadKind::Json(value) => serde_json::from_value(value.clone()).ok(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageHash(pub [u8; 32]);

impl From<MessageHash> for VaaHash {
    fn from(value: MessageHash) -> Self {
        VaaHash(utils::keccak256(value.0))
    }
}

impl fmt::Display for MessageHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Deref for MessageHash {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageHash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<[u8]> for MessageHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for MessageHash {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VaaHash(pub [u8; 32]);

impl fmt::Display for VaaHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl Deref for VaaHash {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VaaHash {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl AsRef<[u8]> for VaaHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for VaaHash {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
