use smallvec::{smallvec, SmallVec};
use std::io::{Read, Result, Seek, SeekFrom, Write};

pub(crate) struct XorRead<T: Read + Seek> {
    inner: T,
}

impl<T: Read + Seek> Read for XorRead<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let spos = self.inner.stream_position()?;
        let n = self.inner.read(buf)?;

        for (idx, byte) in buf.iter_mut().take(n).enumerate() {
            *byte ^= ((spos + idx as u64) & 0xFF) as u8;
        }

        Ok(n)
    }
}

impl<T: Read + Seek> Seek for XorRead<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

pub(crate) trait XorReadable<T: Read + Seek> {
    fn xor_read(self) -> XorRead<T>;
}

impl<T: Read + Seek> XorReadable<T> for T {
    fn xor_read(self) -> XorRead<T> {
        XorRead { inner: self }
    }
}

pub(crate) struct XorWrite<T: Write + Seek> {
    inner: T,
}

impl<T: Write + Seek> Write for XorWrite<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let spos = self.inner.stream_position()?;

        let n = self.inner.write(
            &buf.iter()
                .enumerate()
                .map(|(idx, byte)| byte ^ ((spos + idx as u64) & 0xFF) as u8)
                .collect::<Vec<_>>(),
        )?;

        Ok(n)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl<T: Write + Seek> Seek for XorWrite<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}

pub(crate) trait XorWritable<T: Write + Seek> {
    fn xor_write(self) -> XorWrite<T>;
}

impl<T: Write + Seek> XorWritable<T> for T {
    fn xor_write(self) -> XorWrite<T> {
        XorWrite { inner: self }
    }
}

pub(crate) trait StreamLen<T: Seek> {
    fn stream_len(&mut self) -> Result<u64>;
}

impl<T: Seek> StreamLen<T> for T {
    fn stream_len(&mut self) -> Result<u64> {
        let prev_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;

        if prev_pos != len {
            self.seek(SeekFrom::Start(prev_pos))?;
        }

        Ok(len)
    }
}

pub(crate) fn chunked_copy<R: Read + Sized, W: Write + Sized>(
    bytes: &mut R,
    file: &mut W,
    count: usize,
) -> Result<()> {
    const CHUNK_SIZE: usize = 1048576;

    let mut buf: Vec<u8> = vec![0; CHUNK_SIZE];
    let mut written: usize = 0;

    while written < count {
        let n = bytes.take((count - written) as u64).read(&mut buf[..])?;
        file.write_all(&buf[..n])?;

        written += n;
    }
    Ok(())
}

pub(crate) fn read_fixed<const S: usize, R>(bytes: &mut R) -> Result<[u8; S]>
where
    R: Read + Sized,
{
    let mut buffer: [u8; S] = [0; S];
    bytes.take(S as u64).read_exact(&mut buffer)?;

    Ok(buffer)
}

pub(crate) fn read<R>(bytes: &mut R, size: usize) -> Result<SmallVec<[u8; 128]>>
where
    R: Read + Sized,
{
    let mut buffer: SmallVec<[u8; 128]> = smallvec![0; size];
    bytes.take(buffer.len() as u64).read_exact(&mut buffer)?;

    Ok(buffer)
}
