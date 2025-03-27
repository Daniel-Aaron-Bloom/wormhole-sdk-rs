use wormhole_io::deploys::ChainId;

use crate::{EncodedAmount, Readable, TypePrefixedPayload, Writeable};

use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferWithMessage {
    pub norm_amount: EncodedAmount,
    pub token_address: [u8; 32],
    pub token_chain: ChainId,
    pub redeemer: [u8; 32],
    pub redeemer_chain: ChainId,
    pub sender: [u8; 32],
    pub payload: Vec<u8>,
}

impl TypePrefixedPayload for TransferWithMessage {
    const TYPE: &[u8] = &[3];
}

impl Readable for TransferWithMessage {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        Self: Sized,
        R: io::Read,
    {
        Ok(Self {
            norm_amount: Readable::read(reader)?,
            token_address: Readable::read(reader)?,
            token_chain: Readable::read(reader)?,
            redeemer: Readable::read(reader)?,
            redeemer_chain: Readable::read(reader)?,
            sender: Readable::read(reader)?,
            payload: {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf)?;
                buf
            },
        })
    }
}

impl Writeable for TransferWithMessage {
    fn written_size(&self) -> usize {
        self.norm_amount.written_size()
            + self.token_address.written_size()
            + self.token_chain.written_size()
            + self.redeemer.written_size()
            + self.redeemer_chain.written_size()
            + self.sender.written_size()
            + self.payload.len()
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        Self: Sized,
        W: io::Write,
    {
        self.norm_amount.write(writer)?;
        self.token_address.write(writer)?;
        self.token_chain.write(writer)?;
        self.redeemer.write(writer)?;
        self.redeemer_chain.write(writer)?;
        self.sender.write(writer)?;
        writer.write_all(&self.payload)?;
        Ok(())
    }
}
