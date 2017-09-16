use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::io::{Read, Write, Cursor};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::usize;
use std::fs::OpenOptions;
use std::mem::transmute;

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

#[derive(Clone)]
pub struct BitString {
    data: Vec<u8>,
}

impl BitString {
    pub fn new() -> BitString {
        BitString { data: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn push_bit(&mut self, bit_val: u8) {
        let bit_val = if bit_val != 0 {
            1
        } else {
            0
        };
        self.data.push(bit_val);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i < self.data.len() {
            let c = if self.data[i] == 0 {
                '0'
            } else {
                '1'
            };
            result.push(c);
            i += 1;
        }
        result
    }

    pub fn get(&self, i: usize) -> u8 {
        self.data[i]
    }
}

pub struct BitWriter {
    pub output: Box<Write>,
    next_byte: u8,
    next_bit_index: u8
}

impl BitWriter {
    pub fn wrap(output: Box<Write>) -> BitWriter {
        let next_byte = 0;
        let next_bit_index = 0;
        BitWriter {
            output,
            next_byte,
            next_bit_index
        }
    }
    pub fn append(&mut self, bits: &BitString) {
        let mut i = 0;
        while i < bits.len() {
            self.next_byte = self.next_byte | (bits.get(i) << self.next_bit_index);
            self.next_bit_index += 1;
            if self.next_bit_index > 7 {
                self.write_byte()
            }
            i += 1;
        }
    }

    pub fn flush(&mut self) {
        self.write_byte();
    }

    fn write_byte(&mut self) {
        self.next_bit_index = 0;
        let a: [u8; 1] = [self.next_byte];
        self.output.write(&a);
        self.next_byte = 0;
    }
}

fn byte_string(data: u8, num_bits: u8) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < num_bits {
        let c = if (data & (1 << i)) != 0 {
            '1'
        } else {
            '0'
        };
        result.push(c);
        i += 1;
    }
    result
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
    let mut tree = make_tree(v);
    // print_tree(&tree, 0);
    let table = make_code_table(&tree);
    let p = 97 as u8;
    // println!("{}", table[&p].to_string());
    // move curosr for input file to start
    f.seek(SeekFrom::Start(0));

    // open output file
    let mut output = OpenOptions::new().read(true).write(true).create(true).truncate(true).open("test.txt.hc").expect("could open output file");
    // write to output file the length of the list of byte occurences
    let leaves = make_tree_leaves(list);
    let list_len: u8 = leaves.len() as u8;
    let list_len_buf: [u8; 1] = [list_len; 1];
    output.write_all(&list_len_buf);
    // write to output file the list of byte occurences
    for l in leaves {
        match l {
            Leaf {count, value} => {
                let val: u8 = value;
                let count_bytes: [u8; 8] = unsafe {
                    transmute(count.to_be())
                };                              
                let mut buf: [u8; 9] = [0; 9];
                buf[0] = val;
                buf[1..].clone_from_slice(&count_bytes);
                output.write_all(&buf);
            }
            _ => {}
        }
    }
    // wrap out file in BitWriter
    let mut bw = BitWriter::wrap(Box::new(output));
    // for each byte in the input file write its bits to the output file
    write_compressed_data(&mut f, &mut bw, &table);

    // call BitWriter.flush()
    bw.flush();
    // file should be compressed
}

fn write_compressed_data(input: &mut Read, output: &mut BitWriter, table: &HashMap<u8, BitString>) {
    let mut buf: [u8; 4096] = [0; 4096];
    let mut n = input.read(&mut buf).expect("Error reading from stream");
    while n != 0 {
        {
            let slice = &buf[0..n];
            for b in slice {
                match table.get(b) {
                    Some(bit_str) => {
                        output.append(bit_str);
                    }
                    _ => {
                        panic!("Could not find byte code in table");
                    }
                }
            }
        }
        n = input.read(&mut buf).expect("Error reading from stream");
    }
}

fn make_code_table(tree: &Tree) -> HashMap<u8, BitString> {
    let mut table = HashMap::new();
    let mut bits = BitString::new();
    traverse(tree, bits, &mut table);
    table
}

fn traverse(current: &Tree, mut bits: BitString, table: &mut HashMap<u8, BitString>) {
    match *current {
        Node {count, ref left, ref right} => {
            let mut left_bits = bits.clone();
            let mut right_bits = bits;
            left_bits.push_bit(0);
            right_bits.push_bit(1);
            traverse(&left, left_bits, table);
            traverse(&right, right_bits, table);
        },
        Leaf {count, value}  => {
            table.insert(value, bits);
        }
    }
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