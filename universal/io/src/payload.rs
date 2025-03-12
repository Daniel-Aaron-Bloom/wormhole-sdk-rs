use std::{io, marker::PhantomData};

use array_util::SliceExt;

use crate::{Readable, Writeable};

struct TypeCheckReader<T: TypePrefixedPayload>(PhantomData<T>);

impl<T: TypePrefixedPayload> Readable for TypeCheckReader<T> {
    const SIZE: Option<usize> = Some(T::TYPE.len());

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        // If only it were possible to use `Self::Size` here
        const CHUNK_SIZE: usize = 32;
        let mut id_iter = T::TYPE.array_chunks_ext();
        for id_val in id_iter.by_ref() {
            let mut chunk = [0u8; CHUNK_SIZE];
            reader.read_exact(&mut chunk)?;
            if *id_val != chunk {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid payload type",
                ));
            }
        }
        let id_val = id_iter.remainder();
        let chunk = &mut [0u8; CHUNK_SIZE][..id_val.len()];
        reader.read_exact(chunk)?;
        if id_val != chunk {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid payload type",
            ));
        }
        Ok(Self(PhantomData))
    }
}

/// Trait to capture common payload behavior. We do not recommend overwriting
/// any trait methods. Simply set the type constant and implement [`Readable`]
/// and [`Writeable`].
pub trait TypePrefixedPayload: Readable + Writeable + Clone + std::fmt::Debug {
    const TYPE: &[u8];

    /// Returns the size of the payload, including the type prefix.
    fn payload_written_size(&self) -> usize {
        self.written_size() + Self::TYPE.len()
    }

    /// Read the payload, including the type prefix if applicable.
    fn read_payload<R: io::Read>(reader: &mut R) -> Result<Self, io::Error> {
        TypeCheckReader::<Self>::read(reader)?;
        Readable::read(reader)
    }

    /// Read the payload from a slice. Under the hood, this uses
    /// [read_payload](TypePrefixedPayload::read_payload).
    ///
    /// NOTE: This method will check that the slice is empty after reading the
    /// payload.
    fn read_slice(buf: &[u8]) -> Result<Self, io::Error> {
        let buf = &mut &buf[..];
        let out = Self::read_payload(buf)?;

        if buf.is_empty() {
            Ok(out)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid payload length",
            ))
        }
    }

    /// Write the payload, including the type prefix if applicable.
    fn write_payload<W: io::Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        Self::TYPE.write(writer)?;
        Writeable::write(self, writer)
    }

    /// Write the payload to a vector
    fn to_payload_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.payload_written_size());
        self.write_payload(&mut buf).expect("no alloc failure");
        buf
    }
}

#[cfg(test)]
mod test {
    use crate::{Readable, TypePrefixedPayload, Writeable, WriteableArray, WriteableSequence};
    use hex_literal::hex;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct NineteenBytes([u8; 19]);

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Message {
        pub a: u32,
        pub b: NineteenBytes,
        pub c: WriteableSequence<u32, Vec<u8>>,
        pub d: WriteableArray<u64, 4>,
        pub e: bool,
    }

    impl TypePrefixedPayload for Message {
        const TYPE: &[u8] = &[69];
    }

    impl Readable for Message {
        const SIZE: Option<usize> = Some(88);

        fn read<R>(reader: &mut R) -> std::io::Result<Self>
        where
            Self: Sized,
            R: std::io::Read,
        {
            Ok(Self {
                a: Readable::read(reader)?,
                b: NineteenBytes(Readable::read(reader)?),
                c: Readable::read(reader)?,
                d: Readable::read(reader)?,
                e: Readable::read(reader)?,
            })
        }
    }

    impl Writeable for Message {
        fn written_size(&self) -> usize {
            88
        }

        fn write<W>(&self, writer: &mut W) -> std::io::Result<()>
        where
            W: std::io::Write,
        {
            self.a.write(writer)?;
            self.b.0.write(writer)?;
            self.c.write(writer)?;
            self.d.write(writer)?;
            self.e.write(writer)?;
            Ok(())
        }
    }

    #[test]
    fn to_vec_payload() {
        let msg = Message {
            a: 420,
            b: NineteenBytes(hex!("ba5edba5edba5edba5edba5edba5edba5edba5")),
            c: b"Somebody set us up the bomb.".to_vec().into(),
            d: [0x45; 4].into(),
            e: true,
        };

        let mut encoded = msg.to_payload_vec();
        assert_eq!(encoded, hex!("45000001a4ba5edba5edba5edba5edba5edba5edba5edba50000001c536f6d65626f6479207365742075732075702074686520626f6d622e000000000000004500000000000000450000000000000045000000000000004501"));
        assert_eq!(encoded.capacity(), 1 + msg.written_size());
        assert_eq!(encoded.capacity(), encoded.len());

        let mut cursor = std::io::Cursor::new(&mut encoded);
        let decoded = Message::read_payload(&mut cursor).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn invalid_length() {
        let encoded = hex!("45000001a4ba5edba5edba5edba5edba5edba5edba5edba50000001c536f6d65626f6479207365742075732075702074686520626f6d622e00000000000000450000000000000045000000000000004500000000000000450169");

        assert!(matches!(
            Message::read_slice(&encoded).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData,
        ));
    }

    #[test]
    fn read_slice() {
        let encoded = hex!("45000001a4ba5edba5edba5edba5edba5edba5edba5edba50000001c536f6d65626f6479207365742075732075702074686520626f6d622e000000000000004500000000000000450000000000000045000000000000004501");

        let expected = Message {
            a: 420,
            b: NineteenBytes(hex!("ba5edba5edba5edba5edba5edba5edba5edba5")),
            c: b"Somebody set us up the bomb.".to_vec().into(),
            d: [0x45; 4].into(),
            e: true,
        };

        let decoded = Message::read_slice(&encoded).unwrap();
        assert_eq!(decoded, expected);
    }
}
