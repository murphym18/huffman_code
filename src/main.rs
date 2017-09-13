use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::usize;

#[derive(Eq, PartialEq)]
pub enum Tree {
    Node {
        count: u64,
        left: Box<Tree>,
        right: Box<Tree>
    },
    Leaf {
        count: u64,
        value: u8
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Tree) -> Ordering {
        let other_count = get_count(other);
        let my_count = get_count(self);
        // Notice that the we flip the ordering here
        other_count.cmp(&my_count)
        // other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Tree) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_count(t: &Tree) -> u64 {
    match *t {
        Node {count, ref left, ref right} => count,
        Leaf {count, value} => count
    }
}

fn print_tree(t: &Tree, spaces: u32) {
    match *t {
        Node {count, ref left, ref right} => {
            let mut i = 0;
            while i < spaces {
                print!(" ");
                i = i + 1;
            }
            println!("node");
            print_tree(left, spaces + 2);
            print_tree(right, spaces + 2);
        }
        Leaf {count, value} => {
            let mut i = 0;
            while i < spaces {
                print!(" ");
                i = i + 1;
            }
            println!("{} counted {}", value, count)
        }
    }
}

use Tree::{Node, Leaf};

fn main() {
    let mut f = File::open("test.txt").expect("File not found");
    let list = count_byte_occurrences(&mut f);
    let v = make_tree_leaves(list);
    let mut t = make_tree(v);
    print_tree(&t, 0);
}

fn make_tree(v: Vec<Tree>) -> Tree {
    let mut heap = BinaryHeap::from(v);
    while heap.len() > 1 {
        let l = heap.pop().expect("what???");
        let r = heap.pop().expect("what???");
        let count = get_count(&l) + get_count(&r);
        let left = Box::new(l);
        let right = Box::new(r);
        let n = Node {
            left,
            right,
            count
        };
        heap.push(n);
    }
    heap.pop().expect("what???")
}

fn make_tree_leaves(byte_occurrences: [u64; 256]) -> Vec<Tree> {
    let mut result: Vec<Tree> = Vec::new();
    let mut i: u32 = 0;
    for x in byte_occurrences.iter() {
        if *x != 0 {
            result.push(Leaf{
                count: *x,
                value: i as u8
            });
        }
        i = i + 1;
    }
    result
}

fn count_byte_occurrences(input: &mut Read) -> [u64; 256] {
    let mut byte_counts: [u64; 256] = [0; 256];
    let mut buf: [u8; 4096] = [0; 4096];
    let mut n = input.read(&mut buf).expect("Error reading from stream");
    while n != 0 {
        update_byte_counts(&mut byte_counts, &buf[0..n]);
        n = input.read(&mut buf).expect("Error reading from stream");
    }
    byte_counts
}

fn update_byte_counts(byte_counts: &mut [u64; 256], buf: &[u8]) {
    for b in buf {
        let index = *b as usize;
        byte_counts[index] += 1;
    }
}
// fn make_byte_tree(byte_occurrences: &[u64; 256]) {

// }