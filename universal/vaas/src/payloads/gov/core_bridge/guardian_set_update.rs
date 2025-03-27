use io::WriteableSequence;

use crate::{Readable, TypePrefixedPayload, Writeable};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GuardianSetUpdate {
    _gap: [u8; 2], // This should never be encoded with anything.
    pub new_index: u32,
    pub guardians: WriteableSequence<u8, Vec<[u8; 20]>>,
}

impl TypePrefixedPayload for GuardianSetUpdate {
    const TYPE: &[u8] = &[2];
}

impl Readable for GuardianSetUpdate {
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
        R: std::io::Read,
    {
        let _gap = Readable::read(reader)?;
        if _gap != [0; 2] {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid guardian set update",
            ));
        }

        Ok(Self {
            _gap,
            new_index: Readable::read(reader)?,
            guardians: Readable::read(reader)?,
        })
    }
}

impl Writeable for GuardianSetUpdate {
    fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: std::io::Write,
    {
        self._gap.write(writer)?;
        self.new_index.write(writer)?;

        let guardians = &self.guardians;
        (guardians.len() as u8).write(writer)?;
        for guardian in guardians.iter() {
            guardian.write(writer)?;
        }
        Ok(())
    }

    fn written_size(&self) -> usize {
        2 + 4 + 1 + self.guardians.len() * 20
    }
}
