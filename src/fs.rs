use fuser::{FileAttr, FileType};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time;

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
    FileAttr {
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

pub trait Elem {
    fn get_attr(&self) -> &FileAttr;
    fn to_dir(&self) -> Option<&Dir> {
        None
    }
    fn to_mut_dir(&mut self) -> Option<&mut Dir> {
        None
    }
    fn to_file(&self) -> Option<&File> {
        None
    }
}

// we want a bunch of traits. wrap em up.
pub trait DispElem: fmt::Debug + fmt::Display + Elem {}
impl<T> DispElem for T where T: fmt::Debug + fmt::Display + Elem {}
pub type Kid = Arc<Mutex<Box<dyn DispElem>>>;

#[derive(Debug)]
pub struct Dir {
    attr: FileAttr,
    kids: HashMap<String, Kid>, // strictly tree, no "." or ".."
}

impl Elem for Dir {
    fn get_attr(&self) -> &FileAttr {
        &self.attr
    }
    fn to_dir(&self) -> Option<&Dir> {
        Some(self)
    }
    fn to_mut_dir(&mut self) -> Option<&mut Dir> {
        Some(self)
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kids = self.kids.keys().cloned().collect::<Vec<String>>();
        write!(f, "Dir({:?})", kids)
    }
}

impl Dir {
    fn new(ino: u64) -> Self {
        Dir {
            attr: new_attr(ino, FileType::Directory, DIR_PERM, 2),
            kids: HashMap::new(),
        }
    }

    fn add(&mut self, name: &str, kid: Kid) {
        self.kids.insert(name.to_string(), kid);
    }
}

#[derive(Debug)]
pub struct File {
    attr: FileAttr,
    data: String,
}

impl Elem for File {
    fn get_attr(&self) -> &FileAttr {
        &self.attr
    }
    fn to_file(&self) -> Option<&File> {
        Some(self)
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File({})", self.data)
    }
}

impl File {
    fn new(ino: u64, dat: &str) -> Self {
        File {
            attr: new_attr(ino, FileType::RegularFile, FILE_PERM, 1),
            data: dat.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Fs {
    inode_alloc: u64,
    root: Kid,
}

impl Fs {
    pub fn new() -> Self {
        let dir: Box<dyn DispElem> = Box::new(Dir::new(1));
        let root = Arc::new(Mutex::new(dir));
        Fs {
            inode_alloc: 1,
            root: root,
        }
    }

    pub fn new_test() -> Self {
        let mut fs = Self::new();

        let mut d1 = Box::new(Dir::new(fs.alloc_inode()));
        let f1 = Box::new(File::new(fs.alloc_inode(), "HELLO"));
        d1.add("f1", Arc::new(Mutex::new(f1)));
        let d2 = Box::new(Dir::new(fs.alloc_inode()));

        if let Some(ref mut dir) = fs.root.lock().unwrap().to_mut_dir() {
            dir.add("dir1", Arc::new(Mutex::new(d1)));
            dir.add("dir2", Arc::new(Mutex::new(d2)));
        }

        fs
    }

    fn alloc_inode(&mut self) -> u64 {
        self.inode_alloc += 1;
        self.inode_alloc
    }

    // TODO: return ref not copy...
    pub fn walk(&mut self, comps: Vec<String>) -> Option<Kid> {
        println!("walking {comps:?}");
        let mut parents: Vec<Kid> = Vec::new();
        let mut cur = self.root.clone();
        for comp in comps {
            println!("comp {comp} current {}", cur.lock().unwrap());
            if comp.len() == 0 {
                continue;
            }

            let mut next = None;
            let mut add_parent = false;

            // find out what's next under lock.
            if let Some(dir) = cur.lock().unwrap().to_dir() {
                if comp == "." {
                    // keep cur...
                } else if comp == ".." {
                    if let Some(parent) = parents.pop() {
                        next = Some(parent.clone());
                    }
                } else if let Some(kid) = dir.kids.get(&comp) {
                    add_parent = true;
                    next = Some(kid.clone());
                } else {
                    println!("not found");
                    return None;
                }
            } else {
                println!("cur not dir");
                return None;
            }

            // move to next
            if add_parent {
                parents.push(cur.clone());
            }
            if let Some(next) = next {
                cur = next;
            }
        }
        //println!("found {}", cur.lock().unwrap());
        return Some(cur);
    }

    pub fn test_walk(&mut self, path: &str) -> Option<Kid> {
        let r = self.walk(split_path(path));
        if let Some(ref kid) = r {
            println!("got {}", kid.lock().unwrap());
        }
        println!("");
        r
    }
}
