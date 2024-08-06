#pragma author cyaneko
#pragma description Laby2 Steam version archive file

#pragma pattern_limit 1048576

import std.core;
import std.string;
import std.sys;

struct FileSystemEntry {
    be u32 name_len [[hidden]];
    be char16 name[name_len];
    be u64 file_size;
} [[name(this.name)]];

struct File {
    u8 data[parent.fse[std::core::array_index()].file_size];
} [[name(parent.fse[std::core::array_index()].name)]];

struct Laby2Steam {
	// these bytes don't seem to matter, fwiw replacing them with all zeroes or something else did nothing to the game
    padding[12]; 
	
    be u32 fs_entry_count;    
    FileSystemEntry fse[fs_entry_count];
    File files[fs_entry_count];
};

char magic[4] @ 0x00;
std::assert(magic == "AAFC", "This is not a Laby2 Steam version archive!");

Laby2Steam laby2 @ $;