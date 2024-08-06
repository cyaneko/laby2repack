use crate::arch::*;
use crate::error::*;
use crate::io::chunked_copy;
use crate::parse::*;

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
        let name_len = parse_fixed::<u32, { size_of::<u32>() }, _, _>(input, |buf| {
            let n = u32::from_be_bytes(buf);
            if n == 0 || n > 64 {
                Err(RepackerValueError::PathLenOutOfRange(1, 64, n.into()).into())
            } else {
                Ok(n)
            }
        })?;

        let name = parse::<String, _, _>(input, 2 * name_len as usize, |buf| {
            String::from_utf16be(buf).map_err(|e| RepackerValueError::StringConversion(e).into())
        })?;

        let file_size = parse_fixed::<u64, { size_of::<u64>() }, _, _>(input, |buf| {
            let n = u64::from_be_bytes(buf);
            if name.ends_with("/") && n != 0 {
                Err(RepackerValueError::DirLenNotZero(n).into())
            } else {
                Ok(n)
            }
        })?;

        Ok(Self { name, file_size })
    }
}

impl Laby2 {
    fn read_header_from<R: Read + Seek + Sized>(input: &mut R) -> Result<Self, RepackerError> {
        let header = parse_fixed::<ArchiveHeader, 16, _, _>(input, |buf| {
            if &buf[0..4] != b"AAFC" {
                Err(RepackerValueError::InvalidMagic(buf[0..4].try_into().unwrap()).into())
            } else {
                Ok(ArchiveHeader(buf))
            }
        })?;

        let fs_entry_count = parse_fixed::<u32, { size_of::<u32>() }, _, _>(input, |buf| {
            Ok(u32::from_be_bytes(buf))
        })?;

        let mut files: Vec<ArchiveFile> = Vec::default();
        for _ in 0..fs_entry_count {
            let fs_entry = FileSystemEntry::read_from(input)?;
            files.push(ArchiveFile {
                file_data: fs_entry,
                offset: None,
            });
        }

        let mut stream_location = input.stream_position()?;
        let file_end = input.stream_len()?;
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
