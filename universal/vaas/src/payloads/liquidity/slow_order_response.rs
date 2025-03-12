//! Slow Order Response

use wormhole_io::{Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlowOrderResponse {
    pub base_fee: u64,
}

impl Readable for SlowOrderResponse {
    const SIZE: Option<usize> = u64::SIZE;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        Ok(Self {
            base_fee: Readable::read(reader)?,
        })
    }
}

impl Writeable for SlowOrderResponse {
    fn written_size(&self) -> usize {
        self.base_fee.written_size()
    }
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        Self: Sized,
        W: std::io::Write,
    {
        self.base_fee.write(writer)
    }
}

impl TypePrefixedPayload for SlowOrderResponse {
    const TYPE: &[u8] = &[2];
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn slow_order_response_write() {
        let slow_order_response = SlowOrderResponse {
            base_fee: 1234567890,
        };

        let encoded = slow_order_response.to_payload_vec();
        let expected = hex!("0200000000499602d2");

        assert_eq!(encoded, expected);
    }
}
