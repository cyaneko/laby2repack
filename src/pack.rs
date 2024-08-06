use std::{
    collections::VecDeque,
    env,
    fs::{self, File},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use crate::error::*;
use crate::{arch::*, io::chunked_copy};

impl FileSystemEntry {
    fn write_to<W: Write + Sized>(&self, output: &mut W) -> Result<(), RepackerError> {
        let iter = self.name.encode_utf16();

        let name_len = (iter.clone().count() as u32).to_be_bytes().into_iter();
        let name = iter.flat_map(u16::to_be_bytes);
        let file_size = self.file_size.to_be_bytes().into_iter();

        let data: Vec<u8> = name_len.chain(name).chain(file_size).collect();

        Ok(output.write_all(&data)?)
    }
}

impl Laby2 {
    pub(crate) fn pack<W: Write + Seek + Sized>(
        mut output: W,
        path: &Path,
    ) -> Result<(), RepackerError> {
        // check if directory exists
        if fs::metadata(path)?.is_dir() {
            let pwd = env::current_dir()?;

            // enumerate pointed-to dir for files to repack
            env::set_current_dir(path)?;

            // collect them here
            let mut header: Option<ArchiveHeader> = None;
            let mut files: Vec<FileSystemEntry> = Default::default();

            let mut dir_stack: VecDeque<PathBuf> = VecDeque::from([PathBuf::from(".")]);

            while let Some(dir) = dir_stack.pop_front() {
                // enumerate current directory
                let dir_info = fs::read_dir(dir)?;

                for item in dir_info {
                    let path = item?.path();

                    let path_str = path
                        .clone()
                        .into_os_string()
                        .into_string()
                        .map_err(RepackerValueError::PathConversion)?;
                    let path_str = path_str
                        .clone()
                        .strip_prefix(r".\")
                        .map_or_else(|| path_str, |s| s.to_string())
                        .replace('\\', "/");

                    let item_data = fs::symlink_metadata(&path)?;
                    if item_data.is_dir() {
                        // queue for enumeration
                        dir_stack.push_back(path.clone());

                        // create directory entry
                        files.push(FileSystemEntry {
                            name: path_str + r"/",
                            file_size: 0,
                        });
                    } else if item_data.is_file() {
                        // check if it's the archive header
                        if header.is_none() && path.ends_with(ARCH_HEADER_FILE_NAME) {
                            // read header
                            let mut arch_header: ArchiveHeader = Default::default();
                            File::open(path)?.read_exact(&mut arch_header.0)?;

                            // check if header is valid
                            if &arch_header.0[0..4] != b"AAFC" {
                                return Err(RepackerValueError::InvalidMagic(
                                    arch_header.0[0..4].try_into().unwrap(),
                                )
                                .into());
                            } else {
                                header = Some(arch_header);
                            }
                        } else {
                            // create file entry
                            files.push(FileSystemEntry {
                                name: path_str,
                                file_size: item_data.len(),
                            });
                        }
                    } else {
                        return Err(RepackerError::SymlinkDetected(path));
                    }
                }
            }

            // deleted .header
            if header.is_none() {
                return Err(RepackerError::MissingHeader);
            }

            // sort so that dirs get entered into the archive first
            files.sort_unstable();

            // write header
            output.write_all(&header.unwrap().0)?;

            // write amount of files
            output.write_all(&(files.len() as u32).to_be_bytes())?;

            // pass 1: write metadata
            for f in files.iter() {
                f.write_to(&mut output)?;
            }

            // pass 2: write content
            for FileSystemEntry { name, file_size } in files {
                if !name.ends_with(r"/") {
                    chunked_copy(&mut File::open(r"./".to_owned() + &name)?, &mut output, file_size as usize)?;
                }
            }

            env::set_current_dir(pwd)?;

            Ok(())
        } else {
            Err(RepackerError::NotADir(path.to_path_buf()))
        }
    }
}
