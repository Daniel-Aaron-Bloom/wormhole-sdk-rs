use wormhole_io::deploys::ChainId;

use crate::{Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegisterChain {
    /// This is a placeholder for the `chain` field in the
    /// [`GovernanceHeader`]. The `chain` field is never used for
    /// `RegisterChain` messages. It must be decoded, and should always be
    /// empty.
    ///
    /// [`Governanceheader`]: crate::payloads::gov::GovernanceHeader
    _gap: [u8; 2],
    pub foreign_chain: ChainId,
    pub foreign_emitter: [u8; 32],
}

impl TypePrefixedPayload for RegisterChain {
    const TYPE: &[u8] = &[1];
}

impl Readable for RegisterChain {
    const SIZE: Option<usize> = Some(2 + 2 + 32);

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
        R: std::io::Read,
    {
        let _gap = Readable::read(reader)?;
        if _gap != [0; 2] {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid register chain",
            ));
        }

        Ok(Self {
            _gap,
            foreign_chain: Readable::read(reader)?,
            foreign_emitter: Readable::read(reader)?,
        })
    }
}

impl Writeable for RegisterChain {
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self._gap.write(writer)?;
        self.foreign_chain.write(writer)?;
        self.foreign_emitter.write(writer)
    }

    fn written_size(&self) -> usize {
        <Self as Readable>::SIZE.unwrap()
    }
}
