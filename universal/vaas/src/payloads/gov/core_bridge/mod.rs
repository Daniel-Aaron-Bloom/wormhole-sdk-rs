mod contract_upgrade;
pub use contract_upgrade::ContractUpgrade;

mod guardian_set_update;
pub use guardian_set_update::GuardianSetUpdate;

mod recover_chain_id;
pub use recover_chain_id::RecoverChainId;

mod set_message_fee;
pub use set_message_fee::SetMessageFee;

mod transfer_fees;
pub use transfer_fees::TransferFees;

use crate::{Readable, TypePrefixedPayload, Writeable};
use hex_literal::hex;

/// A.K.A. "Core".
pub const GOVERNANCE_MODULE: [u8; 32] =
    hex!("00000000000000000000000000000000000000000000000000000000436f7265");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Decree {
    ContractUpgrade(ContractUpgrade),
    GuardianSetUpdate(GuardianSetUpdate),
    SetMessageFee(SetMessageFee),
    TransferFees(TransferFees),
    RecoverChainId(RecoverChainId),
}

impl TypePrefixedPayload for Decree {
    const TYPE: &[u8] = &[];
}

impl Writeable for Decree {
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        match self {
            Decree::ContractUpgrade(inner) => inner.write_payload(writer),
            Decree::GuardianSetUpdate(inner) => inner.write_payload(writer),
            Decree::SetMessageFee(inner) => inner.write_payload(writer),
            Decree::TransferFees(inner) => inner.write_payload(writer),
            Decree::RecoverChainId(inner) => inner.write_payload(writer),
        }
    }

    fn written_size(&self) -> usize {
        1 + match self {
            Decree::ContractUpgrade(inner) => inner.written_size(),
            Decree::GuardianSetUpdate(inner) => inner.written_size(),
            Decree::SetMessageFee(inner) => inner.written_size(),
            Decree::TransferFees(inner) => inner.written_size(),
            Decree::RecoverChainId(inner) => inner.written_size(),
        }
    }
}

impl Readable for Decree {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        let decree = match [u8::read(reader)?].as_slice() {
            <ContractUpgrade as TypePrefixedPayload>::TYPE => {
                Decree::ContractUpgrade(Readable::read(reader)?)
            }
            <GuardianSetUpdate as TypePrefixedPayload>::TYPE => {
                Decree::GuardianSetUpdate(Readable::read(reader)?)
            }
            <SetMessageFee as TypePrefixedPayload>::TYPE => {
                Decree::SetMessageFee(Readable::read(reader)?)
            }
            <TransferFees as TypePrefixedPayload>::TYPE => {
                Decree::TransferFees(Readable::read(reader)?)
            }
            <RecoverChainId as TypePrefixedPayload>::TYPE => {
                Decree::RecoverChainId(Readable::read(reader)?)
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid Core Bridge decree",
                ))
            }
        };

        Ok(decree)
    }
}
