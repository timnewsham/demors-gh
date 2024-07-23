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

    fn to_kid(self) -> Kid {
        Arc::new(Mutex::new(Box::new(self)))
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
            data: dat.to_owned(),
        }
    }

    fn to_kid(self) -> Kid {
        Arc::new(Mutex::new(Box::new(self)))
    }
}

#[derive(Debug)]
pub struct Fs {
    inode_alloc: u64,
    root: Kid,
}

impl Fs {
    pub fn new() -> Self {
        Fs {
            inode_alloc: 1,
            root: Dir::new(1).to_kid(),
        }
    }

    fn alloc_inode(&mut self) -> u64 {
        self.inode_alloc += 1;
        self.inode_alloc
    }

    pub fn root(&self) -> Kid {
        self.root.clone()
    }

    pub fn new_file(&mut self, parent: Kid, name: &str, dat: &str) -> Option<Kid> {
        let mut locked = parent.lock().unwrap();
        let dir = locked.to_mut_dir()?;
        let kid = File::new(self.alloc_inode(), dat).to_kid();
        dir.kids.insert(name.to_owned(), kid.clone());
        Some(kid)
    }

    pub fn new_dir(&mut self, parent: Kid, name: &str) -> Option<Kid> {
        let mut locked = parent.lock().unwrap();
        let dir = locked.to_mut_dir()?;
        let kid = Dir::new(self.alloc_inode()).to_kid();
        dir.kids.insert(name.to_owned(), kid.clone());
        Some(kid)
    }

    pub fn walk(&mut self, comps: Vec<String>) -> Option<Kid> {
        // println!("walking {comps:?}");
        let mut parents: Vec<Kid> = Vec::new();
        let mut cur = self.root.clone();
        for comp in comps {
            //println!("comp {comp} current {}", cur.lock().unwrap());
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
                    //println!("not found");
                    return None;
                }
            } else {
                //println!("cur not dir");
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
        let comps = split_path(path);
        println!("walking {path} {comps:?}");
        let r = self.walk(comps);
        if let Some(ref kid) = r {
            println!("got {}", kid.lock().unwrap());
        }
        println!("");
        r
    }

    pub fn show_tree(&mut self) {
        show_tree(self.root(), ".", 0);
    }
}

pub fn show_tree(k: Kid, name: &str, level: usize) {
    if level == 0 {
        println!("Tree:");
    }
    println!(
        "{0:>1$}[{2}] {3}: {4}",
        "",
        level * 2,
        Arc::strong_count(&k),
        name,
        k.lock().unwrap()
    );

    if let Some(dir) = k.lock().unwrap().to_dir() {
        for (nm, kid) in dir.kids.iter() {
            show_tree(kid.clone(), &nm, level + 1);
        }
    }

    if level == 0 {
        println!("");
    }
}
