//! Fast Market Order

use crate::{
    wormhole_io::{wormhole_deploys::ChainId, WriteableSequence},
    Readable, TypePrefixedPayload, Writeable,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastMarketOrder {
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub target_chain: ChainId,
    pub redeemer: [u8; 32],
    pub sender: [u8; 32],
    pub refund_address: [u8; 32],
    pub max_fee: u64,
    pub init_auction_fee: u64,
    pub deadline: u32,
    pub redeemer_message: WriteableSequence<u16, Vec<u8>>,
}

impl Readable for FastMarketOrder {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        Ok(Self {
            amount_in: Readable::read(reader)?,
            min_amount_out: Readable::read(reader)?,
            target_chain: Readable::read(reader)?,
            redeemer: Readable::read(reader)?,
            sender: Readable::read(reader)?,
            refund_address: Readable::read(reader)?,
            max_fee: Readable::read(reader)?,
            init_auction_fee: Readable::read(reader)?,
            deadline: Readable::read(reader)?,
            redeemer_message: Readable::read(reader)?,
        })
    }
}

impl Writeable for FastMarketOrder {
    fn written_size(&self) -> usize {
        self.amount_in.written_size()
            + self.min_amount_out.written_size()
            + self.target_chain.written_size()
            + self.redeemer.written_size()
            + self.sender.written_size()
            + self.refund_address.written_size()
            + self.max_fee.written_size()
            + self.init_auction_fee.written_size()
            + self.deadline.written_size()
            + self.redeemer_message.written_size()
    }
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        Self: Sized,
        W: std::io::Write,
    {
        self.amount_in.write(writer)?;
        self.min_amount_out.write(writer)?;
        self.target_chain.write(writer)?;
        self.redeemer.write(writer)?;
        self.sender.write(writer)?;
        self.refund_address.write(writer)?;
        self.max_fee.write(writer)?;
        self.init_auction_fee.write(writer)?;
        self.deadline.write(writer)?;
        self.redeemer_message.write(writer)?;
        Ok(())
    }
}

impl TypePrefixedPayload for FastMarketOrder {
    const TYPE: &[u8] = &[11];
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn fast_market_order_write() {
        let fast_market_order = FastMarketOrder {
            amount_in: 1234567890,
            min_amount_out: 69420,
            target_chain: 69.into(),
            redeemer: hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"),
            sender: hex!("beefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdead"),
            refund_address: hex!(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ),
            max_fee: 1234567890,
            init_auction_fee: 69420,
            deadline: 420,
            redeemer_message: b"All your base are belong to us.".to_vec().into(),
        };

        let encoded = fast_market_order.to_payload_vec();
        let expected = hex!("0b00000000499602d20000000000010f2c0045deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa00000000499602d20000000000010f2c000001a4001f416c6c20796f75722062617365206172652062656c6f6e6720746f2075732e");

        assert_eq!(encoded, expected);
    }
}
