
use std::collections::HashMap;
use std::fmt;
use std::time;
use fuser::{FileAttr, FileType};

const OWNER_UID: u32 = 0;
const OWNER_GID: u32 = 55;
const DIR_PERM: u16 = 0o550;
const FILE_PERM: u16 = 0o440;

fn split_path(path: &str) -> Vec<String> {
	path.split('/')
		.filter(|comp| comp.len() > 0)
		.map(|comp| comp.to_owned())
		.collect()
}

fn new_attr(ino: u64, kind: FileType, perm: u16, nlink: u32) -> FileAttr {
	let now = time::SystemTime::now();
	FileAttr{
		ino: ino,
		atime: now,
		mtime: now,
		ctime: now,
		crtime: now,
		kind: kind,
		perm: perm,
		nlink: nlink,
		uid: OWNER_UID,
		gid: OWNER_GID,
		blksize: 512,

		size: 0,
		blocks: 0,
		rdev: 0,
		flags: 0,
		padding: 0,
	}
}

#[derive(Debug)]
pub enum DirMember {
	Dir(Dir),
	File(File),
}

impl fmt::Display for DirMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			DirMember::Dir(dir) => write!(f, "Dir({:?})", dir.kids.keys().cloned().collect::<Vec<String>>()),
			DirMember::File(file) => write!(f, "File({})", file.data),
		}
    }
}


#[derive(Debug)]
pub struct Dir {
	attr: FileAttr,
	kids: HashMap<String, DirMember>, // strictly tree, no "." or ".."
}

impl Dir {
	fn new(ino: u64) -> Self {
		Dir{
			attr: new_attr(ino, FileType::Directory, DIR_PERM, 2),
			kids: HashMap::new(),
		}
	}

	fn add_dir(&mut self, name: &str, kid: Dir) {
		self.kids.insert(name.to_string(), DirMember::Dir(kid));
	}
	fn add_file(&mut self, name: &str, kid: File) {
		self.kids.insert(name.to_string(), DirMember::File(kid));
	}
}

#[derive(Debug)]
pub struct File {
	attr: FileAttr,
	data: String,
}

impl File {
	fn new(ino: u64, dat: &str) -> Self {
		File{
			attr: new_attr(ino, FileType::RegularFile, FILE_PERM, 1),
			data: dat.to_string(),
		}
	}
}

#[derive(Debug)]
pub struct Fs {
	inode_alloc: u64,
	root: DirMember, // but always a dir.
}


impl Fs {
	pub fn new() -> Self {
		Fs{
			inode_alloc: 1,
			root: DirMember::Dir(Dir::new(1)),
		}
	}

	pub fn new_test() -> Self {
		let mut fs = Self::new();

		let mut d1 = Dir::new(fs.alloc_inode());
		d1.add_file("f1", File::new(fs.alloc_inode(), "HELLO"));
		let d2 = Dir::new(fs.alloc_inode());

		if let DirMember::Dir(ref mut dir) = fs.root {
			dir.add_dir("dir1", d1);
			dir.add_dir("dir2", d2);
		}

		fs
	}

	fn alloc_inode(&mut self) -> u64 {
		self.inode_alloc += 1;
		self.inode_alloc
	}

	// TODO: return ref not copy...
	pub fn walk(&mut self, comps: Vec<String>) -> bool {
		println!("walking {comps:?}");
		let mut parents = Vec::new();
		let mut cur = &self.root;
		for comp in comps {
			println!("comp {comp} current {cur}");
			if comp.len() == 0 {
				continue;
			}
			if let DirMember::Dir(dir) = cur {
				if comp == "." {
					continue;
				} else if comp == ".." {
					if let Some(parent) = parents.pop() {
						cur = parent;
					}
					continue;
				} else if let Some(kid) = dir.kids.get(&comp) {
					parents.push(cur);
					cur = kid;
				} else {
					println!("not found");
					return false
				}
			} else {
				println!("cur not dir");
				return false
			}
		}
		println!("found {cur}");
		return true
	}

	pub fn test_walk(&mut self, path: &str) -> bool {
		let r = self.walk(split_path(path));
		println!("");
		r
	}
}
