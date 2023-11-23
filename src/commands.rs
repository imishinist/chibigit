use crate::{io_util, Object, ObjectID, ObjectType};

use std::fs;
use std::io::{self, BufRead, Read};

use clap::Args;

#[derive(Args)]
pub struct Init {
    #[arg(short = 'b', long)]
    initial_branch: Option<String>,
}

impl Init {
    pub fn run(&self) {
        if fs::metadata(".git").is_ok() {
            // Do nothing.
            println!("Reinitialized existing Git repository in .git/");
            return;
        }

        let dirs = [
            ".git/objects",
            ".git/objects/pack",
            ".git/objects/info",
            ".git/info",
            ".git/hooks",
            ".git/refs",
            ".git/refs/heads",
            ".git/refs/tags",
            ".git/branches",
        ];
        for dir in &dirs {
            io_util::create_dir(dir).expect("Failed to create directory");
        }

        let first_branch = match &self.initial_branch {
            Some(branch) => branch.as_str(),
            None => "master",
        };
        let head = format!("ref: refs/heads/{}\n", first_branch);
        (|| -> io::Result<()> {
            io_util::write_file(".git/config", b"")?;
            io_util::write_file(".git/HEAD", head.as_bytes())?;
            io_util::write_file(".git/info/exclude", b"")?;
            io_util::write_file(
                ".git/description",
                b"Unnamed repository; edit this file 'description' to name the repository.\n",
            )?;
            Ok(())
        })()
        .expect("Failed to write file");

        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        println!(
            "Initialized empty Git repository in {}/.git/",
            current_dir.as_os_str().to_string_lossy()
        );
    }
}

#[derive(Args)]
pub struct LsFiles {}

impl LsFiles {
    pub fn run(&self) {
        let index_state = super::read_index().unwrap();

        for entry in &index_state.entries {
            let mode = entry.mode;
            let sha1 = &entry.object_id;
            println!("{:o} {} 0\t{}", mode.to_octal(), sha1, entry.name);
        }
    }
}

#[derive(Args)]
pub struct CatFile {
    /// show object type
    #[arg(short = 't', value_name = "object")]
    r#type: Option<String>,

    /// pretty print <object> contents
    #[arg(short = 'p', value_name = "object")]
    preview: Option<String>,
}

impl CatFile {
    pub fn run(&self) {
        if let Some(object) = &self.r#type {
            let object_id = ObjectID::from_hex(object).unwrap();
            let object = crate::read_object(&object_id).unwrap();
            println!("{}", object.r#type);
            return;
        }
        if let Some(object) = &self.preview {
            let object_id = ObjectID::from_hex(object).unwrap();
            let object = crate::read_object(&object_id).unwrap();

            match object.r#type {
                ObjectType::Blob => {
                    print!("{}", std::str::from_utf8(&object.content).unwrap());
                }
                ObjectType::Commit => {
                    print!("{}", std::str::from_utf8(&object.content).unwrap());
                }
                ObjectType::Tree => {
                    let tree_content = crate::parse_tree_content(&object.content).unwrap();
                    for entry in tree_content {
                        println!(
                            "{:06o} {} {}\t{}",
                            entry.mode.to_octal(),
                            entry.r#type,
                            entry.get_sha1(),
                            entry.name
                        );
                    }
                }
            }
            return;
        }
    }
}

#[derive(Args)]
pub struct HashObject {
    /// object type
    #[arg(short = 't', value_name = "type")]
    r#type: Option<ObjectType>,

    /// write object into the object database
    #[arg(short = 'w')]
    write: bool,

    /// read object from standard input
    #[arg(long)]
    stdin: bool,

    /// read file names from standard input
    #[arg(long)]
    stdin_paths: bool,
}

impl HashObject {
    pub fn run(&self) {
        let get_sha1 = |object: &Object, write: bool| -> io::Result<ObjectID> {
            let object_id = if write {
                crate::write_object(object)?
            } else {
                crate::hash_object(object)
            };
            Ok(object_id)
        };
        if self.stdin {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .expect("Failed to read from stdin");

            let object = Object::new(ObjectType::Blob, buffer);
            let sha1 = get_sha1(&object, self.write);
            println!("{}", sha1.unwrap());
        }
        if self.stdin_paths {
            for path in io::stdin().lock().lines() {
                let filename = path.unwrap();
                let content = io_util::read_file(&filename).expect("fatal: unable to hash file");
                let object = Object::new(ObjectType::Blob, content);
                let sha1 = get_sha1(&object, self.write);
                println!("{}", sha1.unwrap());
            }
        }
    }
}
