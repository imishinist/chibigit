pub mod commands;

use std::fmt::Display;
pub use commands::*;

use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use chrono::NaiveDateTime;

#[derive(Debug)]
pub struct IndexState {
    #[allow(dead_code)]
    pub header: IndexHeader,

    pub entries: Vec<IndexEntry>,
}

#[derive(Debug, Default)]
pub struct IndexEntry {
    pub ctime: NaiveDateTime,
    pub mtime: NaiveDateTime,

    pub device: u32,
    pub inode: u32,
    pub mode: u32,

    pub uid: u32,
    pub gid: u32,

    pub size: u32,
    pub sha1: [u8; 20],

    pub name_len: u16,
    pub name: String,
}

impl IndexEntry {
    pub fn get_sha1(&self) -> String {
        sha1_to_hex(&self.sha1)
    }
}

#[derive(Debug, Default)]
pub struct IndexHeader {
    pub signature: [u8; 4],
    pub version: u32,
    pub entries: u32,
}

#[derive(Debug)]
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

pub struct TreeEntry {
    pub mode: u32,
    pub name: String,
    pub sha1: [u8; 20],
}

impl TreeEntry {
    pub fn get_sha1(&self) -> String {
        sha1_to_hex(&self.sha1)
    }

}

fn sha1_to_hex(sha1: &[u8]) -> String {
    sha1.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn parse_object(file: &[u8]) -> io::Result<Object> {
    let mut decoder = flate2::read::ZlibDecoder::new(file);
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

    Ok(Object {
        r#type,
        size,
        content: buf,
    })
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
        let mode = parse_u32(&buf);

        buf.truncate(0);
        cursor.read_until(b'\0', &mut buf)?;
        let name = String::from_utf8_lossy(&buf[..buf.len() - 1]).to_string();

        cursor.read_exact(&mut sha1)?;
        entries.push(TreeEntry { mode, name, sha1 });
    }

    Ok(entries)
}

pub fn parse_index_file(file: &[u8]) -> io::Result<IndexState> {
    let mut cursor = io::Cursor::new(file);
    let header = read_header(&mut cursor)?;

    let mut entries = Vec::new();
    for _ in 0..header.entries {
        let entry = read_index_entry(&mut cursor)?;
        entries.push(entry);
    }
    // skip
    Ok(IndexState { header, entries })
}

fn read_index_entry(cursor: &mut io::Cursor<&[u8]>) -> io::Result<IndexEntry> {
    let mut entry = IndexEntry::default();

    entry.ctime = read_timestamp(cursor)?;
    entry.mtime = read_timestamp(cursor)?;
    entry.device = read_u32(cursor)?;
    entry.inode = read_u32(cursor)?;
    entry.mode = read_u32(cursor)?;
    entry.uid = read_u32(cursor)?;
    entry.gid = read_u32(cursor)?;
    entry.size = read_u32(cursor)?;
    cursor.read_exact(&mut entry.sha1)?;

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

fn read_header(cursor: &mut io::Cursor<&[u8]>) -> io::Result<IndexHeader> {
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

fn read_timestamp(cursor: &mut io::Cursor<&[u8]>) -> io::Result<NaiveDateTime> {
    let seconds = read_u32(cursor)?;
    let nanoseconds = read_u32(cursor)?;

    let datetime = NaiveDateTime::from_timestamp_opt(seconds as i64, nanoseconds).ok_or(
        io::Error::new(io::ErrorKind::InvalidData, "Invalid timestamp"),
    )?;
    Ok(datetime)
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

pub fn read_index() -> io::Result<Vec<u8>> {
    let filename = ".git/index";

    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn read_object(object_id: &str) -> io::Result<Vec<u8>> {
    let filename = format!(".git/objects/{}/{}", &object_id[..2], &object_id[2..]);

    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn create_dir(path: &str) -> io::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn write_file(filename: &str, content: &[u8]) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content)?;
    Ok(())
}