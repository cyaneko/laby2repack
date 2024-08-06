pub(crate) const ARCH_HEADER_FILE_NAME: &str = ".header";

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ArchiveHeader(pub(crate) [u8; 16]);

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FileSystemEntry {
    pub(crate) name: String,   // UTF-16BE
    pub(crate) file_size: u64, // BE
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ArchiveFile {
    pub(crate) file_data: FileSystemEntry,
    pub(crate) offset: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Laby2 {
    pub(crate) header: ArchiveHeader,
    pub(crate) files: Vec<ArchiveFile>, // file offset from stream start
}
