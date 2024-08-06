use std::{ffi::OsString, io, path, string};

#[derive(Debug, thiserror::Error)]
pub(crate) enum RepackerValueError {
    #[error("string conversion error")]
    StringConversion(#[from] string::FromUtf16Error),
    #[error("path conversion error")]
    PathConversion(OsString),
    #[error("path len out of range; expected {0}-{1}, got {2}")]
    PathLenOutOfRange(u64, u64, u64),
    #[error("got dir path with non-zero file size of {0}")]
    DirLenNotZero(u64),
    #[error("invalid magic bytes; expected [0x41, 0x41, 0x46, 0x43] (AAFC), got {0:04X?}")]
    InvalidMagic([u8; 4]),
    #[error("missing file offset for file \"{0}\"")]
    MissingOffset(path::PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RepackerError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("missing .header file")]
    MissingHeader,
    #[error("stream read mismatch; expected {0} bytes, got {1}")]
    StreamRead(u64, u64),
    #[error("item at path \"{0}\" is a symlink")]
    SymlinkDetected(path::PathBuf),
    #[error("path \"{0}\" does not point to a directory")]
    NotADir(path::PathBuf),
    #[error("value error: {0}")]
    Value(#[from] RepackerValueError),
}
