use std::io;

use crate::{Readable, TypePrefixedPayload, Writeable};

mod message;

pub mod gov;
pub mod liquidity;
pub mod token_bridge;

pub use message::Message;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(untagged)
)]
#[non_exhaustive]
pub enum PayloadKind {
    Binary(Vec<u8>),
    #[cfg(feature = "serde")]
    Json(serde_json::Value),
}

impl Readable for PayloadKind {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        let mut buf = vec![];
        reader.read_to_end(&mut buf)?;
        Ok(Self::Binary(buf))
    }
}

impl Writeable for PayloadKind {
    fn written_size(&self) -> usize {
        #[allow(unreachable_patterns)]
        match self {
            PayloadKind::Binary(buf) => buf.len(),
            _ => 0,
        }
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        #[allow(unreachable_patterns)]
        match self {
            Self::Binary(buf) => writer.write_all(buf),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tried to write a JSON payload",
            )),
        }
    }
}

impl TypePrefixedPayload for PayloadKind {
    const TYPE: &[u8] = &[];
}
