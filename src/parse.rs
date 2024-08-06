use smallvec::{smallvec, SmallVec};
use std::io::Read;

use crate::error::*;

pub(crate) fn parse_fixed<T, const S: usize, R, O>(bytes: &mut R, op: O) -> Result<T, RepackerError>
where
    R: Read + Sized,
    O: FnOnce([u8; S]) -> Result<T, RepackerError>,
{
    let mut buffer: [u8; S] = [0; S];
    bytes.take(S as u64).read_exact(&mut buffer)?;

    op(buffer)
}

pub(crate) fn parse<T, R, O>(bytes: &mut R, size: usize, op: O) -> Result<T, RepackerError>
where
    R: Read + Sized,
    O: FnOnce(&[u8]) -> Result<T, RepackerError>,
{
    let mut buffer: SmallVec<[u8; 128]> = smallvec![0; size];
    bytes.take(buffer.len() as u64).read_exact(&mut buffer)?;

    op(&buffer)
}
