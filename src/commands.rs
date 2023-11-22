use std::fs;
use clap::Args;
use crate::write_file;

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
            fs::create_dir_all(dir).expect("Failed to create directory");
        }

        let first_branch = match &self.initial_branch {
            Some(branch) => branch.as_str(),
            None => "master",
        };
        let head = format!("ref: refs/heads/{}\n", first_branch);
        (|| -> std::io::Result<()> {
            write_file(".git/config", b"")?;
            write_file(".git/HEAD", head.as_bytes())?;
            write_file(".git/info/exclude", b"")?;
            write_file(".git/description", b"Unnamed repository; edit this file 'description' to name the repository.\n")?;
            Ok(())
        })().expect("Failed to write file");

        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        println!("Initialized empty Git repository in {}/.git/", current_dir.as_os_str().to_string_lossy());
    }
}

#[derive(Args)]
pub struct LsFiles {
}

impl LsFiles {
    pub fn run(&self) {
        let index_file = super::read_index().unwrap();
        let index_state = super::parse_index_file(&index_file).unwrap();

        for entry in &index_state.entries {
            let mode = entry.mode;
            let sha1 = entry.get_sha1();
            println!("{:o} {} 0\t{}", mode, sha1, entry.name);
        }
    }
}

#[derive(Args)]
pub struct CatFile {
    /// pretty print <object> contents
    #[arg(short = 'p', value_name="object")]
    preview: Option<String>,
}

impl CatFile {
    pub fn run(&self) {
        if let Some(object) = &self.preview {
            let object_file = crate::read_object(object).unwrap();
            let object = crate::parse_object(object_file).unwrap();

            match object.r#type {
                crate::ObjectType::Blob => {
                    print!("{}", std::str::from_utf8(&object.content).unwrap());
                },
                crate::ObjectType::Commit => {
                    print!("{}", std::str::from_utf8(&object.content).unwrap());
                },
                crate::ObjectType::Tree => {
                    let tree_content = crate::parse_tree_content(&object.content).unwrap();
                    for entry in tree_content {
                        let entry_object = crate::read_object(&entry.get_sha1()).unwrap();
                        let object = crate::parse_object(entry_object).unwrap();

                        println!("{:06} {} {}\t{}", entry.mode, object.r#type, entry.get_sha1(), entry.name);
                    }
                },
            }
        }
    }
}
