//! Traits for reading/writing Minecraft-encoded values.

use anyhow::{anyhow, bail, Context};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use encoding::{all::UTF_16BE, EncoderTrap, Encoding};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fmt::Display,
    io::{self, Cursor, Read, Write},
    iter::{self, FromIterator},
    marker::PhantomData,
    num::TryFromIntError,
};
use thiserror::Error;
/// Trait implemented for types which can be read
/// from a buffer.
pub trait Readable {
    /// Reads this type from the given buffer.
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized;
}

/// Trait implemented for types which can be written
/// to a buffer.
pub trait Writeable: Sized {
    /// Writes this value to the given buffer.
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()>;
}

impl<'a, T> Writeable for &'a T
where
    T: Writeable,
{
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        T::write(*self, buffer)?;
        Ok(())
    }
}

/// Error when reading a value.
#[derive(Debug, Error)]
pub enum Error {
    #[error("unexpected end of input: failed to read value of type `{0}`")]
    UnexpectedEof(&'static str),
}

macro_rules! integer_impl {
    ($($int:ty, $read_fn:tt, $write_fn:tt),* $(,)?) => {
        $(
            impl Readable for $int {
                fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self> {
                    buffer.$read_fn::<BigEndian>().map_err(anyhow::Error::from)
                }
            }

            impl Writeable for $int {
                fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
                    buffer.$write_fn::<BigEndian>(*self)?;
                    Ok(())
                }
            }
        )*
    }
}

integer_impl! {
    u16, read_u16, write_u16,
    u32, read_u32, write_u32,
    u64, read_u64, write_u64,

    i16, read_i16, write_i16,
    i32, read_i32, write_i32,
    i64, read_i64, write_i64,

    f32, read_f32, write_f32,
    f64, read_f64, write_f64,
}

impl Readable for u8 {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        buffer.read_u8().map_err(anyhow::Error::from)
    }
}

impl Writeable for u8 {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.write_u8(*self)?;
        Ok(())
    }
}
impl Readable for i8 {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        buffer.read_i8().map_err(anyhow::Error::from)
    }
}

impl Writeable for i8 {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.write_i8(*self)?;
        Ok(())
    }
}

impl<T> Readable for Option<T>
where
    T: Readable,
{
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // Assume boolean prefix.
        let present = bool::read(buffer)?;

        if present {
            Ok(Some(T::read(buffer)?))
        } else {
            Ok(None)
        }
    }
}

impl<T> Writeable for Option<T>
where
    T: Writeable,
{
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let present = self.is_some();
        present.write(buffer)?;

        if let Some(value) = self {
            value.write(buffer)?;
        }

        Ok(())
    }
}
/// A variable-length integer as defined by the Minecraft protocol.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VarInt(pub i32);

impl Readable for VarInt {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            let read = u8::read(buffer)?;
            let value = i32::from(read & 0b0111_1111);
            result |= value.overflowing_shl(7 * num_read).0;

            num_read += 1;

            if num_read > 5 {
                bail!(
                    "VarInt too long (max length: 5, value read so far: {})",
                    result
                );
            }
            if read & 0b1000_0000 == 0 {
                break;
            }
        }
        Ok(VarInt(result))
    }
}

impl TryFrom<VarInt> for usize {
    type Error = TryFromIntError;
    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}

impl From<usize> for VarInt {
    fn from(x: usize) -> Self {
        VarInt(x as i32)
    }
}

impl From<VarInt> for i32 {
    fn from(x: VarInt) -> Self {
        x.0
    }
}

impl From<i32> for VarInt {
    fn from(x: i32) -> Self {
        VarInt(x)
    }
}

impl VarInt {
    pub fn write_to(&self, mut writer: impl Write) -> io::Result<()> {
        let mut x = self.0 as u32;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }

            writer.write_all(&[temp])?;

            if x == 0 {
                break;
            }
        }
        Ok(())
    }
}

impl Writeable for VarInt {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        self.write_to(buffer).expect("write to Vec failed");
        Ok(())
    }
}
impl Readable for String {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let length = 64;

        // Read string into buffer.
        let mut temp = vec![0u8; length];
        buffer
            .read_exact(&mut temp)
            .map_err(|_| Error::UnexpectedEof("String"))?;
        let s = std::str::from_utf8(&temp).context("string contained invalid UTF8")?;
        let s = s.trim_end_matches(' ');
        Ok(s.to_owned())
    }
}

impl Writeable for String {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let mut us = self.clone();
        if us.len() > 64 {
            bail!("String too long!");
        }
        for _ in 0..64 - us.len() {
            us.push_str(" ");
        }
        buffer.extend_from_slice(us.as_bytes());

        Ok(())
    }
}

impl Readable for bool {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let x = u8::read(buffer)?;

        if x == 0 {
            Ok(false)
        } else if x == 1 {
            Ok(true)
        } else {
            Err(anyhow::anyhow!("invalid boolean tag {}", x))
        }
    }
}

impl Writeable for bool {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let x = if *self { 1u8 } else { 0 };
        x.write(buffer)?;

        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct ByteArray(pub [u8; 1024]);
impl Readable for ByteArray {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut arr = [0; 1024];
        buffer.read_exact(&mut arr)?;
        Ok(Self(arr))
    }
}
impl Writeable for ByteArray {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.extend_from_slice(&self.0);
        Ok(())
    }
}
pub const MAX_LENGTH: usize = 1024 * 1024; // 2^20 elements

/// Reads and writes an array of inner `Writeable`s.
/// The array is prefixed with a `VarInt` length.
///
/// This will reject arrays of lengths larger than MAX_LENGTH.
pub struct LengthPrefixedVec<'a, P, T>(pub Cow<'a, [T]>, PhantomData<P>)
where
    [T]: ToOwned<Owned = Vec<T>>;

impl<'a, P, T> Readable for LengthPrefixedVec<'a, P, T>
where
    T: Readable,
    [T]: ToOwned<Owned = Vec<T>>,
    P: TryInto<usize> + Readable,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let length: usize = P::read(buffer)?.try_into()?;

        if length > MAX_LENGTH {
            bail!("array length too large ({} > {})", length, MAX_LENGTH);
        }

        let vec = iter::repeat_with(|| T::read(buffer))
            .take(length)
            .collect::<anyhow::Result<Vec<T>>>()?;
        Ok(Self(Cow::Owned(vec), PhantomData))
    }
}

impl<'a, P, T> Writeable for LengthPrefixedVec<'a, P, T>
where
    T: Writeable,
    [T]: ToOwned<Owned = Vec<T>>,
    P: TryFrom<usize> + Writeable,
    P::Error: std::error::Error + Send + Sync + 'static,
{
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        P::try_from(self.0.len())?.write(buffer)?;
        self.0
            .iter()
            .for_each(|item| item.write(buffer).expect("failed to write to vec"));

        Ok(())
    }
}

impl<'a, P, T> From<LengthPrefixedVec<'a, P, T>> for Vec<T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(x: LengthPrefixedVec<'a, P, T>) -> Self {
        x.0.into_owned()
    }
}

impl<'a, P, T> From<&'a [T]> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(slice: &'a [T]) -> Self {
        Self(Cow::Borrowed(slice), PhantomData)
    }
}

impl<'a, P, T> From<Vec<T>> for LengthPrefixedVec<'a, P, T>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(vec: Vec<T>) -> Self {
        Self(Cow::Owned(vec), PhantomData)
    }
}

pub type ShortPrefixedVec<'a, T> = LengthPrefixedVec<'a, u16, T>;

/// A vector of bytes which consumes all remaining bytes in this packet.
/// This is used by the plugin messaging packets, for one.
pub struct LengthInferredVecU8<'a>(pub Cow<'a, [u8]>);

impl<'a> Readable for LengthInferredVecU8<'a> {
    fn read(buffer: &mut Cursor<&[u8]>) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut vec = Vec::new();
        buffer.read_to_end(&mut vec)?;
        Ok(LengthInferredVecU8(Cow::Owned(vec)))
    }
}

impl<'a> Writeable for LengthInferredVecU8<'a> {
    fn write(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        buffer.extend_from_slice(&*self.0);
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for LengthInferredVecU8<'a> {
    fn from(slice: &'a [u8]) -> Self {
        LengthInferredVecU8(Cow::Borrowed(slice))
    }
}

impl<'a> From<LengthInferredVecU8<'a>> for Vec<u8> {
    fn from(x: LengthInferredVecU8<'a>) -> Self {
        x.0.into_owned()
    }
}
