use std::{
    array, io, iter,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

use wormhole_deploys::ChainId;

#[cfg(feature = "alloy")]
use alloy_primitives::{Address, FixedBytes, Uint};

pub trait Readable: Sized {
    const SIZE: Option<usize>;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read;
}

pub trait Writeable {
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write;

    fn written_size(&self) -> usize;

    fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.written_size());
        self.write(&mut buf).expect("no alloc failure");
        buf
    }
}

impl Readable for u8 {
    const SIZE: Option<usize> = Some(1);

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Writeable for u8 {
    fn written_size(&self) -> usize {
        <Self as Readable>::SIZE.unwrap()
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(&[*self])
    }
}

impl Readable for bool {
    const SIZE: Option<usize> = Some(1);

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        match u8::read(reader)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid bool value",
            )),
        }
    }
}

impl Writeable for bool {
    fn written_size(&self) -> usize {
        <Self as Readable>::SIZE.unwrap()
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(&[u8::from(*self)])
    }
}

macro_rules! impl_for_int {
    ($($type:ty),+ $(,)?) => {$(
        impl Readable for $type {
            const SIZE: Option<usize> = Some(std::mem::size_of::<$type>());

            fn read<R>(reader: &mut R) -> io::Result<Self>
            where
                R: io::Read,
            {
                let mut buf = [0u8; std::mem::size_of::<$type>()];
                reader.read_exact(&mut buf)?;
                Ok(Self::from_be_bytes(buf))
            }
        }

        impl Writeable for $type {
            fn written_size(&self) -> usize {
                <Self as Readable>::SIZE.unwrap()
            }

            fn write<W>(&self, writer: &mut W) -> io::Result<()>
            where
                W: io::Write,
            {
                writer.write_all(&self.to_be_bytes())
            }
        }
    )+};
}
impl_for_int! {u16, u32, u64, u128, i8, i16, i32, i64, i128}

impl<const N: usize> Readable for [u8; N] {
    const SIZE: Option<usize> = Some(N);

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let mut buf = [0; N];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl<const N: usize> Writeable for [u8; N] {
    fn written_size(&self) -> usize {
        <u8 as Readable>::SIZE.unwrap()
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(self)
    }
}

macro_rules! impl_for_array {
    ($($type:ty),+ $(,)?) => {$(
        impl<const N: usize> Readable for [$type; N] {
            const SIZE: Option<usize> = match <$type>::SIZE {
                Some(t) => Some(t * N),
                None => None,
            };

            fn read<R>(reader: &mut R) -> io::Result<Self>
            where
                R: io::Read,
            {
                let mut buf = [Default::default(); N];
                for i in 0..N {
                    buf[i] = <$type>::read(reader)?;
                }
                Ok(buf)
            }
        }

        impl<const N: usize> Writeable for [$type; N] {
            fn written_size(&self) -> usize {
                self.iter().map(|s| s.written_size()).sum()
            }
            fn write<W>(&self, writer: &mut W) -> io::Result<()>
            where
                W: io::Write,
            {
                for i in 0..N {
                    self[i].write(writer)?;
                }
                Ok(())
            }
        }
    )+};
}

impl_for_array! {bool, u16, u32, u64, u128, i8, i16, i32, i64, i128}

impl<T: Writeable> Writeable for &'static [T] {
    fn written_size(&self) -> usize {
        self.iter().map(|s| s.written_size()).sum()
    }
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        for v in self.iter() {
            v.write(writer)?;
        }
        Ok(())
    }
}

impl<T> Readable for Option<T>
where
    T: Readable,
{
    const SIZE: Option<usize> = None;
    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        match bool::read(reader)? {
            true => Ok(Some(T::read(reader)?)),
            false => Ok(None),
        }
    }
}

impl<T> Writeable for Option<T>
where
    T: Writeable,
{
    fn written_size(&self) -> usize {
        match self {
            Some(value) => true.written_size() + value.written_size(),
            None => false.written_size(),
        }
    }
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        match self {
            Some(value) => {
                true.write(writer)?;
                value.write(writer)
            }
            None => false.write(writer),
        }
    }
}

/// Wrapper for `Vec<u8>` or similar. Encoding is similar to Borsh, where the length is encoded as `u32` by default.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct WriteableArray<T, const LEN: usize>(pub [T; LEN]);

impl<T: Default, const LEN: usize> Default for WriteableArray<T, LEN> {
    fn default() -> Self {
        Self::new(array::from_fn(|_| T::default()))
    }
}

impl<T, const LEN: usize> WriteableArray<T, LEN> {
    pub const fn new(value: [T; LEN]) -> Self {
        WriteableArray(value)
    }
    pub fn into_inner(self) -> [T; LEN] {
        self.0
    }
}

impl<T, const LEN: usize> From<[T; LEN]> for WriteableArray<T, LEN> {
    fn from(value: [T; LEN]) -> Self {
        Self::new(value)
    }
}

impl<T, const LEN: usize> Deref for WriteableArray<T, LEN> {
    type Target = [T; LEN];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const LEN: usize> DerefMut for WriteableArray<T, LEN> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Readable, const LEN: usize> Readable for WriteableArray<T, LEN> {
    const SIZE: Option<usize> = match T::SIZE {
        Some(t) => Some(t * LEN),
        None => None,
    };
    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        struct Collector<T, const N: usize>([T; N]);

        impl<T, const N: usize> FromIterator<T> for Collector<T, N> {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                let mut iter = iter.into_iter();
                let res: [_; N] = std::array::from_fn(|_| iter.next().unwrap());
                Collector(res)
            }
        }
        iter::repeat_with(|| Readable::read(reader))
            .collect::<io::Result<Collector<_, LEN>>>()
            .map(|v| WriteableArray(v.0))
    }
}

impl<T: Writeable, const LEN: usize> Writeable for WriteableArray<T, LEN> {
    fn written_size(&self) -> usize {
        self.0.iter().map(|s| s.written_size()).sum()
    }

    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        for s in self.iter() {
            s.write(writer)?;
        }
        Ok(())
    }
}

impl Readable for ChainId {
    const SIZE: Option<usize> = u16::SIZE;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        u16::read(reader).map(Self::from_u16)
    }
}

impl Writeable for ChainId {
    fn written_size(&self) -> usize {
        self.to_u16().written_size()
    }
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        self.to_u16().write(writer)
    }
}

/// Wrapper for `Vec<u8>` or similar. Encoding is similar to Borsh, where the length is encoded as `u32` by default.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator>,
    Sequence: ?Sized,
{
    phantom: PhantomData<Length>,
    sequence: Sequence,
}

impl<Length, Sequence> WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator>,
{
    pub fn new(sequence: Sequence) -> Self {
        Self {
            sequence,
            phantom: PhantomData,
        }
    }

    pub fn into_inner(self) -> Sequence {
        self.sequence
    }
}

impl<Length, Sequence> WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator>,
    Sequence: ?Sized,
{
    pub fn from_ref(sequence: &Sequence) -> &Self {
        // This is safe because of repr(transparent)
        unsafe { mem::transmute(sequence) }
    }
    pub fn try_encoded_len(&self) -> io::Result<Length> {
        match self.sequence.into_iter().len().try_into() {
            Ok(len) => Ok(len),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "L overflow when converting from usize",
            )),
        }
    }
}

impl<Length, Sequence> From<Sequence> for WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator>,
{
    fn from(value: Sequence) -> Self {
        Self::new(value)
    }
}

impl<Length, Sequence> std::ops::Deref for WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator>,
    Sequence: ?Sized,
{
    type Target = Sequence;

    fn deref(&self) -> &Self::Target {
        &self.sequence
    }
}

impl<Length, Sequence> std::ops::DerefMut for WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator>,
    Sequence: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sequence
    }
}

impl<Length, Sequence> Readable for WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    Length: Readable + TryInto<usize>,
    for<'a> &'a Sequence:
        IntoIterator<IntoIter: ExactSizeIterator, Item: Deref<Target: Sized + Readable>>,
    Sequence: for<'a> FromIterator<<<&'a Sequence as IntoIterator>::Item as Deref>::Target>,
{
    const SIZE: Option<usize> = None;

    fn read<R>(reader: &mut R) -> io::Result<Self>
    where
        R: io::Read,
    {
        let len = Length::read(reader)?;
        let len = len.try_into().map_err(|_e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "length too out of bounds for usize",
            )
        })?;
        (0..len)
            .map(|_| Readable::read(reader))
            .collect::<io::Result<Sequence>>()
            .map(Self::new)
    }
}

impl<Length, Sequence> Writeable for WriteableSequence<Length, Sequence>
where
    usize: TryInto<Length>,
    Length: Writeable,
    for<'a> &'a Sequence: IntoIterator<IntoIter: ExactSizeIterator, Item: Deref<Target: Writeable>>,
    Sequence: ?Sized,
{
    fn written_size(&self) -> usize {
        self.sequence.into_iter().map(|s| s.written_size()).sum()
    }
    fn write<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        let len = self.try_encoded_len()?;
        len.write(writer)?;
        for s in self.sequence.into_iter() {
            s.write(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloy")]
const _: () = {
    impl<const N: usize> Readable for FixedBytes<N> {
        const SIZE: Option<usize> = Some(N);

        fn read<R>(reader: &mut R) -> io::Result<Self>
        where
            R: io::Read,
        {
            let mut buf = [0; N];
            reader.read_exact(&mut buf)?;
            Ok(Self(buf))
        }
    }

    impl<const N: usize> Writeable for FixedBytes<N> {
        fn written_size(&self) -> usize {
            <Self as Readable>::SIZE.unwrap()
        }

        fn write<W>(&self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
        {
            writer.write_all(&self.0)
        }
    }

    impl<const BITS: usize, const LIMBS: usize> Readable for Uint<BITS, LIMBS> {
        const SIZE: Option<usize> = Some(BITS.div_ceil(8));

        fn read<R>(reader: &mut R) -> io::Result<Self>
        where
            R: io::Read,
        {
            let mut buf = [0u8; BITS];
            let buf = &mut buf[0..BITS.div_ceil(8)];
            reader.read_exact(buf)?;
            Ok(Uint::from_be_slice(buf))
        }
    }

    impl<const BITS: usize, const LIMBS: usize> Writeable for Uint<BITS, LIMBS> {
        fn written_size(&self) -> usize {
            <Self as Readable>::SIZE.unwrap()
        }

        fn write<W>(&self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
        {
            let mut buf = [0u8; BITS];
            let buf = &mut buf[0..BITS.div_ceil(8)];
            self.copy_be_bytes_to(buf);
            writer.write_all(buf)
        }
    }

    impl Readable for Address {
        const SIZE: Option<usize> = Some(20);

        fn read<R>(reader: &mut R) -> io::Result<Self>
        where
            R: io::Read,
        {
            FixedBytes::<20>::read(reader).map(Self)
        }
    }

    impl Writeable for Address {
        fn written_size(&self) -> usize {
            <Self as Readable>::SIZE.unwrap()
        }

        fn write<W>(&self, writer: &mut W) -> io::Result<()>
        where
            W: io::Write,
        {
            self.0.write(writer)
        }
    }
};

#[cfg(test)]
pub mod test {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn u8_read_write() {
        const EXPECTED_SIZE: usize = 1;

        let value = 69u8;

        let mut encoded = Vec::<u8>::with_capacity(EXPECTED_SIZE);
        let mut writer = std::io::Cursor::new(&mut encoded);
        value.write(&mut writer).unwrap();

        let expected = hex!("45");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn u64_read_write() {
        const EXPECTED_SIZE: usize = 8;

        let value = 69u64;
        let mut encoded = Vec::<u8>::with_capacity(EXPECTED_SIZE);
        let mut writer = std::io::Cursor::new(&mut encoded);
        value.write(&mut writer).unwrap();

        let expected = hex!("0000000000000045");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn u8_array_read_write() {
        let data = [1, 2, 8, 16, 32, 64, 69u8];

        let mut encoded = Vec::<u8>::with_capacity(data.len());
        let mut writer = std::io::Cursor::new(&mut encoded);
        data.write(&mut writer).unwrap();

        let expected = hex!("01020810204045");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn u64_array_read_write() {
        let data = WriteableArray::<u64, 7>::new([1, 2, 8, 16, 32, 64, 69u64]);
        const EXPECTED_SIZE: usize = 56;

        let mut encoded = Vec::<u8>::with_capacity(EXPECTED_SIZE);
        let mut writer = std::io::Cursor::new(&mut encoded);
        data.write(&mut writer).unwrap();

        let expected = hex!("0000000000000001000000000000000200000000000000080000000000000010000000000000002000000000000000400000000000000045");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn variable_bytes_read_write_u8() {
        let data = b"All your base are belong to us.";
        let bytes = WriteableSequence::<u8, Vec<u8>>::new(data.to_vec());

        let mut encoded = Vec::<u8>::with_capacity(1 + data.len());
        let mut writer = std::io::Cursor::new(&mut encoded);
        bytes.write(&mut writer).unwrap();

        let expected = hex!("1f416c6c20796f75722062617365206172652062656c6f6e6720746f2075732e");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn variable_bytes_read_write_u16() {
        let data = b"All your base are belong to us.";
        let bytes = WriteableSequence::<u16, Vec<u8>>::new(data.to_vec());

        let mut encoded = Vec::<u8>::with_capacity(2 + data.len());
        let mut writer = std::io::Cursor::new(&mut encoded);
        bytes.write(&mut writer).unwrap();

        let expected = hex!("001f416c6c20796f75722062617365206172652062656c6f6e6720746f2075732e");
        assert_eq!(encoded, expected);

        let mut reader = std::io::Cursor::new(&mut encoded);
        let decoded: WriteableSequence<u16, Vec<u8>> =
            Readable::read(&mut reader).expect("read failed");

        assert_eq!(bytes, decoded);
    }

    #[test]
    fn variable_bytes_read_write_u32() {
        let data = b"All your base are belong to us.";
        let bytes = WriteableSequence::<u32, [u8]>::from_ref(data);

        let mut encoded = Vec::<u8>::with_capacity(4 + data.len());
        let mut writer = std::io::Cursor::new(&mut encoded);
        bytes.write(&mut writer).unwrap();

        let expected =
            hex!("0000001f416c6c20796f75722062617365206172652062656c6f6e6720746f2075732e");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn option_some() {
        let value = Some(69u64);

        let mut encoded = Vec::<u8>::with_capacity(1 + 8);
        let mut writer = std::io::Cursor::new(&mut encoded);
        value.write(&mut writer).unwrap();

        let expected = hex!("010000000000000045");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn option_none() {
        let value: Option<[u8; 64]> = None;

        let mut encoded = Vec::<u8>::with_capacity(1);
        let mut writer = std::io::Cursor::new(&mut encoded);
        value.write(&mut writer).unwrap();

        let expected = hex!("00");
        assert_eq!(encoded, expected);
    }
}
