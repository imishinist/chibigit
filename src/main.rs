use chrono::NaiveDateTime;
use std::fs::File;
use std::io::{self, Read};

#[derive(Debug)]
struct IndexState {
    #[allow(dead_code)]
    header: IndexHeader,

    entries: Vec<IndexEntry>,
}

#[derive(Debug, Default)]
struct IndexEntry {
    ctime: NaiveDateTime,
    mtime: NaiveDateTime,

    device: u32,
    inode: u32,
    mode: u32,

    uid: u32,
    gid: u32,

    size: u32,
    sha1: [u8; 20],

    name_len: u16,
    name: String,
}

#[derive(Debug, Default)]
struct IndexHeader {
    signature: [u8; 4],
    version: u32,
    entries: u32,
}

fn main() {
    let index_file = read_index().unwrap();
    let index_state = parse_file(index_file).unwrap();

    for entry in &index_state.entries {
        let mode = entry.mode;
        let sha1 = sha1_to_hex(&entry.sha1);
        println!("{:o} {} 0\t{}", mode, sha1, entry.name);
    }
}

fn sha1_to_hex(sha1: &[u8]) -> String {
    sha1.iter().map(|b| format!("{:02x}", b)).collect()
}

fn parse_file(file: Vec<u8>) -> io::Result<IndexState> {
    let mut cursor = io::Cursor::new(file);
    let header = parse_header(&mut cursor)?;

    let mut entries = Vec::new();
    for _ in 0..header.entries {
        let entry = parse_index_entry(&mut cursor)?;
        entries.push(entry);
    }
    // skip
    Ok(IndexState { header, entries })
}

fn parse_index_entry(cursor: &mut io::Cursor<Vec<u8>>) -> io::Result<IndexEntry> {
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

    entry.name_len = read_u16(cursor)?;
    let mut name_buffer = vec![0; entry.name_len as usize];
    cursor.read_exact(&mut name_buffer)?;
    entry.name = String::from_utf8(name_buffer).unwrap();

    let floor = (entry.name_len - 2) / 8;
    let target = (floor + 1) * 8 + 2;
    let padding = target - entry.name_len;
    cursor.set_position(cursor.position() + padding as u64);

    Ok(entry)
}

fn parse_header(cursor: &mut io::Cursor<Vec<u8>>) -> io::Result<IndexHeader> {
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

fn read_timestamp(cursor: &mut io::Cursor<Vec<u8>>) -> io::Result<NaiveDateTime> {
    let seconds = read_u32(cursor)?;
    let nanoseconds = read_u32(cursor)?;

    let datetime = NaiveDateTime::from_timestamp_opt(seconds as i64, nanoseconds).ok_or(
        io::Error::new(io::ErrorKind::InvalidData, "Invalid timestamp"),
    )?;
    Ok(datetime)
}

fn read_u16(cursor: &mut io::Cursor<Vec<u8>>) -> io::Result<u16> {
    let mut buffer = [0; 2];
    cursor.read_exact(&mut buffer)?;
    Ok(u16::from_be_bytes(buffer))
}

fn read_u32(cursor: &mut io::Cursor<Vec<u8>>) -> io::Result<u32> {
    let mut buffer = [0; 4];
    cursor.read_exact(&mut buffer)?;
    Ok(u32::from_be_bytes(buffer))
}

fn read_index() -> io::Result<Vec<u8>> {
    let filename = ".git/index";

    let mut file = File::open(filename)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}
