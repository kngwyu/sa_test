use std::error::Error;
use std::io::BufReader;
use std::io::prelude::*;
use std::str::{self, FromStr};

pub struct Scanner<R> {
    bufread: BufReader<R>,
    id: usize,
    buf: Vec<u8>,
}
impl<R: Read> Scanner<R> {
    pub fn new(resource: R) -> Scanner<R> {
        Scanner {
            bufread: BufReader::new(resource),
            id: 0,
            buf: Vec::new(),
        }
    }
    fn next_line(&mut self) -> Option<String> {
        let mut res = String::new();
        match self.bufread.read_line(&mut res) {
            Ok(0) => None,
            Ok(_) => Some(res),
            Err(why) => panic!("error in read_line: {}", why.description()),
        }
    }
    pub fn next<T: FromStr>(&mut self) -> Option<T> {
        while self.buf.is_empty() {
            self.buf = match self.next_line() {
                Some(r) => {
                    self.id = 0;
                    r.trim().as_bytes().to_owned()
                }
                None => return None,
            };
        }
        let l = self.id;
        assert_ne!(self.buf[l], b' ');
        let n = self.buf.len();
        let mut r = l;
        while r < n && self.buf[r] != b' ' {
            r += 1;
        }
        let res = match str::from_utf8(&self.buf[l..r]).ok().unwrap().parse::<T>() {
            Ok(s) => Some(s),
            Err(_) => {
                panic!(
                    "parse error, {:?}",
                    String::from_utf8(self.buf[l..r].to_owned())
                )
            }
        };
        while r < n && self.buf[r] == b' ' {
            r += 1;
        }
        if r == n {
            self.buf.clear();
        } else {
            self.id = r;
        }
        res
    }
    pub fn ne<T: FromStr>(&mut self) -> T {
        self.next::<T>().unwrap()
    }
}
