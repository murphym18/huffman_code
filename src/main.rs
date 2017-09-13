use std::fs::File;
use std::io::Read;

enum Tree {
    Node {
        count: u64,
        left: Box<Tree>,
        right: Box<Tree>
    },
    Leaf {
        count: u64,
        value: u8
    },
    Nil
}

use Tree::{Node, Leaf, Nil};
fn main() {
    let mut f = File::open("test.txt").expect("File not found");
    let list = count_byte_occurrences(&mut f);
    let v = make_tree_leaves(list);
    
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
        let index: usize = *b as usize;
        byte_counts[index] = byte_counts[index] + 1;
    }
}

fn make_tree_leaves(byte_occurrences: [u64; 256]) -> Vec<Tree> {
    let mut result: Vec<Tree> = Vec::new();
    let mut i = 0;
    for x in byte_occurrences.iter() {
        if *x != 0 {
            result.push(Leaf{
                count: *x,
                value: i
            });
        }
        i = i + 1;
    }
    result
}

// fn make_byte_tree(byte_occurrences: &[u64; 256]) {

// }