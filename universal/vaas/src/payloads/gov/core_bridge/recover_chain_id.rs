use wormhole_io::deploys::ChainId;

use crate::{utils::U256, Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecoverChainId {
    pub recovered_chain: ChainId,
    pub evm_chain_id: U256,
    pub new_chain: ChainId,
}

impl TypePrefixedPayload for RecoverChainId {
    const TYPE: &[u8] = &[5];
}

impl Readable for RecoverChainId {
    const SIZE: Option<usize> = Some(2 + 32 + 2);

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
        R: std::io::Read,
    {
        Ok(Self {
            recovered_chain: Readable::read(reader)?,
            evm_chain_id: Readable::read(reader)?,
            new_chain: Readable::read(reader)?,
        })
    }
}

impl Writeable for RecoverChainId {
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self.recovered_chain.write(writer)?;
        self.evm_chain_id.write(writer)?;
        self.new_chain.write(writer)
    }

    fn written_size(&self) -> usize {
        <Self as Readable>::SIZE.unwrap()
    }
}
