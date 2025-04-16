//! A CCTP deposit transfer with message

use crate::{io::WriteableSequence, Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CctpDeposit {
    pub token_address: [u8; 32],
    pub amount: [u8; 32],
    pub source_cctp_domain: u32,
    pub destination_cctp_domain: u32,
    pub cctp_nonce: u64,
    pub burn_source: [u8; 32],
    pub mint_recipient: [u8; 32],
    pub payload: WriteableSequence<u16, Vec<u8>>,
}

impl Readable for CctpDeposit {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        Ok(Self {
            token_address: Readable::read(reader)?,
            amount: Readable::read(reader)?,
            source_cctp_domain: Readable::read(reader)?,
            destination_cctp_domain: Readable::read(reader)?,
            cctp_nonce: Readable::read(reader)?,
            burn_source: Readable::read(reader)?,
            mint_recipient: Readable::read(reader)?,
            payload: Readable::read(reader)?,
        })
    }
}

impl Writeable for CctpDeposit {
    fn written_size(&self) -> usize {
        self.token_address.written_size()
            + self.amount.written_size()
            + self.source_cctp_domain.written_size()
            + self.destination_cctp_domain.written_size()
            + self.cctp_nonce.written_size()
            + self.burn_source.written_size()
            + self.mint_recipient.written_size()
            + self.payload.written_size()
    }
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        Self: Sized,
        W: std::io::Write,
    {
        self.token_address.write(writer)?;
        self.amount.write(writer)?;
        self.source_cctp_domain.write(writer)?;
        self.destination_cctp_domain.write(writer)?;
        self.cctp_nonce.write(writer)?;
        self.burn_source.write(writer)?;
        self.mint_recipient.write(writer)?;
        self.payload.write(writer)?;
        Ok(())
    }
}

impl TypePrefixedPayload for CctpDeposit {
    const TYPE: &[u8] = &[1];
}
