use std::io;

use wormhole_io::{deploys::ChainId, WriteableSequence};

use crate::{Readable, TypePrefixedPayload, Writeable};

impl TypePrefixedPayload for Message {
    const TYPE: &[u8] = &[0xbb];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub version: u8,
    pub message_ty: u8,
    pub index: u64,
    pub target_chain: ChainId,
    pub target: WriteableSequence<u16, Vec<u8>>,
    pub sender: WriteableSequence<u16, Vec<u8>>,
    pub body: WriteableSequence<u16, Vec<u8>>,
}

impl Readable for Message {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let version = Readable::read(reader)?;
        let message_ty = Readable::read(reader)?;
        let index = Readable::read(reader)?;
        let target_chain = Readable::read(reader)?;
        let target = Readable::read(reader)?;
        let sender = Readable::read(reader)?;
        let body = Readable::read(reader)?;
        Ok(Self {
            version,
            message_ty,
            index,
            target_chain,
            sender,
            target,
            body,
        })
    }
}

impl Writeable for Message {
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.version.write(writer)?;
        self.message_ty.write(writer)?;
        self.index.write(writer)?;
        self.target_chain.write(writer)?;
        (self.target.len() as u16).write(writer)?;
        writer.write_all(&self.target)?;
        (self.sender.len() as u16).write(writer)?;
        writer.write_all(&self.sender)?;
        (self.body.len() as u16).write(writer)?;
        writer.write_all(&self.body)?;
        Ok(())
    }

    fn written_size(&self) -> usize {
        1 + 8 + 2 + 2 + self.sender.len() + 2 + self.target.len() + 2 + self.body.len()
    }
}

#[cfg(test)]
mod test {
    use hex_literal::hex;

    use super::*;
    #[test]
    fn it_roundtrips() {
        let message = hex!(
            "0000000000000000000012340002567800147fa9385be102ac3eac297483dd6233d62b3e149600029abc"
        );

        let message = Message::read(&mut &message[..]).unwrap();
        dbg!(&message);

        assert_eq!(message.index, 0);
        assert_eq!(message.target_chain, 0x1234);
        assert_eq!(*message.target, hex!("5678"));
        assert_eq!(
            *message.sender,
            hex!("7fa9385be102ac3eac297483dd6233d62b3e1496")
        );
        assert_eq!(*message.body, hex!("9abc"));
    }
}
