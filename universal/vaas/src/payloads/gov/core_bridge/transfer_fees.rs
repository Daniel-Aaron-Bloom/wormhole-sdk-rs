use wormhole_io::deploys::ChainId;

use crate::{utils::U256, Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransferFees {
    pub chain: ChainId,
    pub amount: U256,
    pub recipient: [u8; 32],
}

impl TypePrefixedPayload for TransferFees {
    const TYPE: &[u8] = &[4];
}

impl Readable for TransferFees {
    const SIZE: Option<usize> = Some(2 + 32 + 32);

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
        R: std::io::Read,
    {
        Ok(Self {
            chain: Readable::read(reader)?,
            amount: Readable::read(reader)?,
            recipient: Readable::read(reader)?,
        })
    }
}

impl Writeable for TransferFees {
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.chain.write(writer)?;
        self.amount.write(writer)?;
        self.recipient.write(writer)
    }

    fn written_size(&self) -> usize {
        <Self as Readable>::SIZE.unwrap()
    }
}
