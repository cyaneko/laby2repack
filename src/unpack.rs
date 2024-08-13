use crate::arch::*;
use crate::error::*;
use crate::io::*;

use std::{
    cmp::Ordering,
    env,
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
    path::{Path, PathBuf},
};

impl FileSystemEntry {
    fn read_from<R: Read + Sized>(input: &mut R) -> Result<Self, RepackerError> {
        let buf = read_fixed::<{ size_of::<u32>() }, _>(input)?;
        let name_len = {
            let n = u32::from_be_bytes(buf);
            if n == 0 || n > 64 {
                Err(RepackerValueError::PathLenOutOfRange(1, 64, n.into()))
            } else {
                Ok(n)
            }
        }?;

        let buf = read(input, 2 * name_len as usize)?;
        let name: String = {
            let combined = buf
                .chunks_exact(2)
                .map(|ch| u16::from_be_bytes(ch.try_into().unwrap()))
                .collect::<Vec<_>>();

            String::from_utf16(&combined).map_err(RepackerValueError::StringConversion)
        }?;

        let buf = read_fixed::<{ size_of::<u64>() }, _>(input)?;
        let file_size = {
            let n = u64::from_be_bytes(buf);
            if name.ends_with('/') && n != 0 {
                Err(RepackerValueError::DirLenNotZero(n))
            } else {
                Ok(n)
            }
        }?;

        Ok(Self { name, file_size })
    }
}

impl Laby2 {
    fn read_header_from<R: Read + Seek + Sized>(input: &mut R) -> Result<Self, RepackerError> {
        let buf = read_fixed::<16, _>(input)?;
        let header = {
            if &buf[0..4] != b"AAFC" {
                Err(RepackerValueError::InvalidMagic(
                    buf[0..4].try_into().unwrap(),
                ))
            } else {
                Ok(ArchiveHeader(buf))
            }
        }?;

        let buf = read_fixed::<{ size_of::<u32>() }, _>(input)?;
        let fs_entry_count = u32::from_be_bytes(buf);

        let mut files: Vec<ArchiveFile> = Vec::default();
        for _ in 0..fs_entry_count {
            let fs_entry = FileSystemEntry::read_from(input)?;
            files.push(ArchiveFile {
                file_data: fs_entry,
                offset: None,
            });
        }

        let file_end = <R as StreamLen<_>>::stream_len(input)?;
        let mut stream_location = input.stream_position()?;
        for ArchiveFile { file_data, offset } in &mut files {
            *offset = Some(stream_location);
            stream_location += file_data.file_size;
        }

        match stream_location.cmp(&file_end) {
            Ordering::Equal => Ok(Self { header, files }),
            _ => Err(RepackerError::StreamRead(stream_location, file_end)),
        }
    }

    pub(crate) fn unpack<R: Read + Seek + Sized>(
        mut input: R,
        path: &Path,
    ) -> Result<(), RepackerError> {
        let pwd = env::current_dir()?;

        fs::create_dir_all(path)?;
        env::set_current_dir(path)?;

        let arch = Self::read_header_from(&mut input)?;

        let mut header_file = File::create(ARCH_HEADER_FILE_NAME)?;
        header_file.write_all(&arch.header.0)?;

        for ArchiveFile { file_data, offset } in &arch.files {
            let path = PathBuf::from(&file_data.name);
            if file_data.name.ends_with('/') {
                fs::create_dir_all(path)?;
            } else {
                input.seek(SeekFrom::Start(
                    offset.ok_or_else(|| RepackerValueError::MissingOffset(path.clone()))?,
                ))?;
                chunked_copy(
                    &mut input,
                    &mut File::create(path)?,
                    file_data.file_size as usize,
                )?;
            }
        }

        env::set_current_dir(pwd)?;

        Ok(())
    }
}
