//! Fill

use wormhole_io::{Readable, TypePrefixedPayload, Writeable, WriteableSequence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub source_chain: u16,
    pub order_sender: [u8; 32],
    pub redeemer: [u8; 32],
    pub redeemer_message: WriteableSequence<u16, Vec<u8>>,
}

impl Readable for Fill {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        Ok(Self {
            source_chain: Readable::read(reader)?,
            order_sender: Readable::read(reader)?,
            redeemer: Readable::read(reader)?,
            redeemer_message: Readable::read(reader)?,
        })
    }
}

impl Writeable for Fill {
    fn written_size(&self) -> usize {
        self.source_chain.written_size()
            + self.order_sender.written_size()
            + self.redeemer.written_size()
            + self.redeemer_message.written_size()
    }

    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        Self: Sized,
        W: std::io::Write,
    {
        self.source_chain.write(writer)?;
        self.order_sender.write(writer)?;
        self.redeemer.write(writer)?;
        self.redeemer_message.write(writer)?;
        Ok(())
    }
}

impl TypePrefixedPayload for Fill {
    const TYPE: &[u8] = &[1];
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn fill_write() {
        let fill = Fill {
            source_chain: 69,
            order_sender: hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"),
            redeemer: hex!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            redeemer_message: b"All your base are belong to us.".to_vec().into(),
        };

        let encoded = fill.to_payload_vec();
        let expected = hex!("010045deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa001f416c6c20796f75722062617365206172652062656c6f6e6720746f2075732e");

        assert_eq!(encoded, expected);
    }
}
