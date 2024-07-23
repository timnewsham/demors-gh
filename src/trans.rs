use std::cmp;

// Trans is a trasaction.
#[derive(Debug)]
pub struct Trans {
    args: Vec<Vec<u8>>,
    resp: Vec<u8>,
}

impl Trans {
    pub fn new() -> Self {
        Trans {
            args: Vec::new(),
            resp: Vec::new(),
        }
    }

    pub fn arg_mode(&self) -> bool {
        return self.resp.len() == 0;
    }

    pub fn add_arg(&mut self, dat: Vec<u8>) {
        // TODO error if not in arg mode
        self.args.push(dat);
    }

    // take_args takes all the args if there are at least n, returning the first n.
    pub fn take_args(&mut self, n: usize) -> Option<Vec<Vec<u8>>> {
        if self.args.len() >= n {
            let mut args = std::mem::take(&mut self.args);
            args.truncate(n);
            Some(args)
        } else {
            None
        }
    }

    pub fn set_resp(&mut self, dat: Vec<u8>) {
        // TODO error if not in arg mode
        self.resp.extend(dat);
    }

    // read_resp takes up to n bytes from the response.
    // this could be more efficient.
    pub fn read_resp(&mut self, n: usize) -> Vec<u8> {
        let n = cmp::min(n, self.resp.len());
        let (hd, tl) = self.resp.split_at(n);
        let res = hd.to_vec();
        self.resp = tl.to_vec();
        return res;
    }
}
