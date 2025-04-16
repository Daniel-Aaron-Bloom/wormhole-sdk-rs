//! Fill

use wormhole_io::{deploys::ChainId, WriteableSequence};

use crate::{Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastFill {
    pub fill_amount: u64,
    pub source_chain: ChainId,
    pub order_sender: [u8; 32],
    pub redeemer: [u8; 32],
    pub redeemer_message: WriteableSequence<u16, Vec<u8>>,
}

impl Readable for FastFill {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        Ok(Self {
            fill_amount: Readable::read(reader)?,
            source_chain: Readable::read(reader)?,
            order_sender: Readable::read(reader)?,
            redeemer: Readable::read(reader)?,
            redeemer_message: Readable::read(reader)?,
        })
    }
}

impl Writeable for FastFill {
    fn written_size(&self) -> usize {
        self.fill_amount.written_size()
            + self.source_chain.written_size()
            + self.order_sender.written_size()
            + self.redeemer.written_size()
            + self.redeemer_message.written_size()
    }

    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        Self: Sized,
        W: std::io::Write,
    {
        self.fill_amount.write(writer)?;
        self.source_chain.write(writer)?;
        self.order_sender.write(writer)?;
        self.redeemer.write(writer)?;
        self.redeemer_message.write(writer)?;
        Ok(())
    }
}

impl TypePrefixedPayload for FastFill {
    const TYPE: &[u8] = &[12];
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn fill_write() {
        let fill = FastFill {
            fill_amount: 420,
            source_chain: 69.into(),
            order_sender: hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"),
            redeemer: hex!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            redeemer_message: b"All your base are belong to us.".to_vec().into(),
        };

        let encoded = fill.to_payload_vec();
        let expected = hex!("0c00000000000001a40045deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa001f416c6c20796f75722062617365206172652062656c6f6e6720746f2075732e");

        assert_eq!(encoded, expected);
    }
}
