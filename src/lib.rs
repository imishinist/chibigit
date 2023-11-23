pub mod commands;
pub use commands::*;

pub(crate) mod io_util;

use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};

use chrono::NaiveDateTime;
use clap::ValueEnum;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Directory,
    File,
    Executable,
    Symlink,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::File
    }
}

impl Mode {
    pub fn to_octal(&self) -> u32 {
        match self {
            Mode::Directory => 0o040_000,
            Mode::File => 0o100_644,
            Mode::Executable => 0o100_755,
            Mode::Symlink => 0o120_000,
        }
    }

    pub fn from_octal(mode: u32) -> Self {
        match mode {
            0o040_000 => Mode::Directory,
            0o100_644 => Mode::File,
            0o100_755 => Mode::Executable,
            0o120_000 => Mode::Symlink,
            _ => panic!("Invalid mode"),
        }
    }
}

#[derive(Debug, Default)]
pub struct ObjectID {
    inner: [u8; 20],
}

impl ObjectID {
    pub fn new(sha1: [u8; 20]) -> Self {
        Self { inner: sha1 }
    }

    pub fn from_hex(hex: &str) -> io::Result<Self> {
        let mut sha1 = [0; 20];

        let mut i = 0;
        for j in (0..40).step_by(2) {
            sha1[i] = u8::from_str_radix(&hex[j..j + 2], 16).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid hex: {}", e))
            })?;
            i += 1;
        }
        Ok(Self { inner: sha1 })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_object_id_from_hex() {
        let hex = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
        let object_id = super::ObjectID::from_hex(hex).unwrap();
        assert_eq!(hex, object_id.to_string());
    }
}

impl Display for ObjectID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = sha1_to_hex(&self.inner);
        write!(f, "{}", s)
    }
}

#[derive(Debug, Default)]
pub struct IndexHeader {
    pub signature: [u8; 4],
    pub version: u32,
    pub entries: u32,
}

#[derive(Debug, Default)]
pub struct IndexEntry {
    pub ctime: NaiveDateTime,
    pub mtime: NaiveDateTime,

    pub device: u32,
    pub inode: u32,
    pub mode: Mode,

    pub uid: u32,
    pub gid: u32,

    pub size: u32,
    pub object_id: ObjectID,

    pub name_len: u16,
    pub name: String,
}

#[derive(Debug)]
pub struct IndexState {
    #[allow(dead_code)]
    pub header: IndexHeader,

    pub entries: Vec<IndexEntry>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ObjectType {
    Commit,
    Tree,
    Blob,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Commit => write!(f, "commit"),
            ObjectType::Tree => write!(f, "tree"),
            ObjectType::Blob => write!(f, "blob"),
        }
    }
}

#[derive(Debug)]
pub struct Object {
    pub r#type: ObjectType,
    pub size: u32,
    pub content: Vec<u8>,
}

impl Object {
    pub fn new(r#type: ObjectType, content: Vec<u8>) -> Self {
        let size = content.len() as u32;
        Self {
            r#type,
            size,
            content,
        }
    }
}

pub fn serialize_object(object: &Object) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(format!("{} {}\0", object.r#type, object.size).as_bytes());
    buffer.extend_from_slice(&object.content);

    buffer
}

pub struct TreeEntry {
    pub r#type: ObjectType,
    mode: Mode,
    pub name: String,
    object_id: ObjectID,
}

impl TreeEntry {
    pub fn new(mode: Mode, name: String, object_id: ObjectID) -> Self {
        let r#type = match mode {
            Mode::Directory => ObjectType::Tree,
            Mode::File => ObjectType::Blob,
            Mode::Executable => ObjectType::Blob,
            Mode::Symlink => ObjectType::Blob,
        };
        Self {
            r#type,
            mode,
            name,
            object_id,
        }
    }

    pub fn get_sha1(&self) -> String {
        self.object_id.to_string()
    }
}

fn sha1_to_hex(sha1: &[u8]) -> String {
    sha1.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn hash_content(content: &[u8]) -> ObjectID {
    let mut hasher = sha1::Sha1::new();
    hasher.update(content);
    let sha1 = hasher.finalize().into();

    ObjectID::new(sha1)
}

pub fn hash_object(object: &Object) -> ObjectID {
    let buffer = serialize_object(object);
    hash_content(&buffer)
}

fn read_u16(cursor: &mut io::Cursor<&[u8]>) -> io::Result<u16> {
    let mut buffer = [0; 2];
    cursor.read_exact(&mut buffer)?;
    Ok(u16::from_be_bytes(buffer))
}

fn read_u32(cursor: &mut io::Cursor<&[u8]>) -> io::Result<u32> {
    let mut buffer = [0; 4];
    cursor.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
}

fn parse_u32(input: &[u8]) -> u32 {
    let mut result = 0;
    for byte in input {
        if *byte < b'0' || *byte > b'9' {
            break;
        }
        result = result * 10 + (byte - b'0') as u32;
    }
    result
}

fn parse_u32_octal(input: &[u8]) -> u32 {
    let mut result = 0;
    for byte in input {
        if *byte < b'0' || *byte > b'7' {
            break;
        }
        result = result * 8 + (byte - b'0') as u32;
    }
    result
}

fn read_timestamp(cursor: &mut io::Cursor<&[u8]>) -> io::Result<NaiveDateTime> {
    let seconds = read_u32(cursor)?;
    let nanoseconds = read_u32(cursor)?;

    let datetime = NaiveDateTime::from_timestamp_opt(seconds as i64, nanoseconds).ok_or(
        io::Error::new(io::ErrorKind::InvalidData, "Invalid timestamp"),
    )?;
    Ok(datetime)
}

fn parse_object_content(content: &[u8]) -> io::Result<Object> {
    let mut decoder = flate2::read::ZlibDecoder::new(content);
    let mut inflated_content = Vec::new();
    decoder.read_to_end(&mut inflated_content)?;

    let mut cursor = io::Cursor::new(inflated_content);

    let mut buf = Vec::new();
    cursor.read_until(b' ', &mut buf)?;
    let r#type = match buf.as_slice() {
        b"commit " => ObjectType::Commit,
        b"tree " => ObjectType::Tree,
        b"blob " => ObjectType::Blob,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid type")),
    };

    buf.truncate(0);
    cursor.read_until(b'\0', &mut buf)?;
    let size = parse_u32(&buf);

    buf.truncate(0);
    cursor.read_to_end(&mut buf)?;

    assert!(buf.len() == size as usize, "Invalid size");
    Ok(Object::new(r#type, buf))
}

pub fn parse_tree_content(content: &[u8]) -> io::Result<Vec<TreeEntry>> {
    let mut cursor = io::Cursor::new(content);

    let mut buf = Vec::new();
    let mut sha1 = [0; 20];
    let mut entries = Vec::new();
    while cursor.position() < content.len() as u64 {
        // "mode" " " "name" "\0" "sha1"
        buf.truncate(0);
        cursor.read_until(b' ', &mut buf)?;
        let mode = Mode::from_octal(parse_u32_octal(&buf));

        buf.truncate(0);
        cursor.read_until(b'\0', &mut buf)?;
        let name = String::from_utf8_lossy(&buf[..buf.len() - 1]).to_string();

        cursor.read_exact(&mut sha1)?;

        let object_id = ObjectID::new(sha1.clone());
        entries.push(TreeEntry::new(mode, name, object_id));
    }

    Ok(entries)
}

fn read_index_header(cursor: &mut io::Cursor<&[u8]>) -> io::Result<IndexHeader> {
    let mut header = IndexHeader::default();
    cursor.read_exact(&mut header.signature)?;
    if &header.signature != b"DIRC" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid signature",
        ));
    }
    header.version = read_u32(cursor)?;
    if header.version != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid version",
        ));
    }
    header.entries = read_u32(cursor)?;

    Ok(header)
}

fn read_index_entry(cursor: &mut io::Cursor<&[u8]>) -> io::Result<IndexEntry> {
    let mut entry = IndexEntry::default();

    entry.ctime = read_timestamp(cursor)?;
    entry.mtime = read_timestamp(cursor)?;
    entry.device = read_u32(cursor)?;
    entry.inode = read_u32(cursor)?;
    entry.mode = Mode::from_octal(read_u32(cursor)?);
    entry.uid = read_u32(cursor)?;
    entry.gid = read_u32(cursor)?;
    entry.size = read_u32(cursor)?;

    let mut sha1 = [0; 20];
    cursor.read_exact(&mut sha1)?;
    entry.object_id = ObjectID::new(sha1);

    let flags = read_u16(cursor)?;
    // skip 4 bytes.
    // 1bit: assume valid
    // 1bit: extended flag (must be zero in version 2)
    // 2bit: stage (during merge)

    entry.name_len = flags & 0xFFF;
    let mut name_buffer = vec![0; entry.name_len as usize];
    cursor.read_exact(&mut name_buffer)?;
    entry.name = String::from_utf8(name_buffer).unwrap();

    let floor = (entry.name_len - 2) / 8;
    let target = (floor + 1) * 8 + 2;
    let padding = target - entry.name_len;
    cursor.set_position(cursor.position() + padding as u64);

    Ok(entry)
}

fn parse_index_content(content: &[u8]) -> io::Result<IndexState> {
    let mut cursor = io::Cursor::new(content);
    let header = read_index_header(&mut cursor)?;

    let mut entries = Vec::new();
    for _ in 0..header.entries {
        let entry = read_index_entry(&mut cursor)?;
        entries.push(entry);
    }
    // skip
    Ok(IndexState { header, entries })
}

pub fn read_index() -> io::Result<IndexState> {
    let content = io_util::read_file(".git/index")?;
    parse_index_content(&content)
}

fn object_filedir(id: &ObjectID) -> String {
    format!(".git/objects/{:02x}", id.inner[0])
}
fn object_filename(id: &ObjectID) -> String {
    let s = id.to_string();
    format!(".git/objects/{}/{}", &s[..2], &s[2..])
}

pub fn read_object(object_id: &ObjectID) -> io::Result<Object> {
    let filename = object_filename(object_id);
    let content = io_util::read_file(&filename)?;
    parse_object_content(&content)
}

pub fn write_object(object: &Object) -> io::Result<ObjectID> {
    let buffer = serialize_object(object);
    let object_id = hash_content(&buffer);

    // mkdir -p .git/objects/xx
    let filename = object_filedir(&object_id);
    io_util::create_dir(&filename)?;

    let filename = object_filename(&object_id);
    if io_util::file_exists(&filename) {
        return Ok(object_id);
    }

    let mut file = File::create(filename)?;
    let mut encoder = ZlibEncoder::new(&mut file, Compression::default());
    encoder.write_all(&buffer)?;

    Ok(object_id)
}
