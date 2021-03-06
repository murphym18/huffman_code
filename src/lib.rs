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

struct BitReader {
    input: Box<Read>,
    buf: u8,
    next_bit: u8
}

impl BitReader {
    fn new(input: Box<Read>) -> BitReader {
        let buf = 0;
        let next_bit = 8;
        BitReader {
            input,
            buf,
            next_bit
        }
    }

    fn next(&mut self) -> u8 {
        if self.next_bit > 7 {
            self.buf = read_one_byte(&mut self.input);
            self.next_bit = 0;
        }

        let result: u8 = (self.buf >> self.next_bit) & 1;
        self.next_bit = self.next_bit + 1;
        result
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



pub fn decompress() {
    let mut compressed_file = File::open("test.txt.hc").expect("File not found");
    // read the first byte
    let table_size = read_one_byte(&mut compressed_file);
    // read the table
    let mut table_vec: Vec<Tree> = Vec::new();
    let mut i = 0;
    while i < table_size {
        table_vec.push(read_one_table_record(&mut compressed_file));
        i = i + 1;
    }
    // make the tree
    let mut tree = make_tree(table_vec);
    print_tree(&tree, 0);

    // find out how big the decompressed file will be
    let size = read_u64_be(&mut compressed_file);
    println!("size of uncompresed file will be {}", size);
    let mut br = BitReader::new(Box::new(compressed_file));

    let mut output = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("tmp.txt")
        .expect("could open output file");

    let mut num_written = 0;
    let mut current_node = &tree;
    while num_written < size {
        match *current_node {
            Leaf {count, value} => {
                output.write(&[value]);
                num_written = num_written + 1;
                current_node = &tree;
            },
            Node {count, ref left, ref right} => {
                let bit = br.next();
                if bit == 0 {
                    current_node = left;
                } else {
                    current_node = right;
                }
            }
        }
    }

}

fn read_u64_be(input: &mut Read) -> u64 {
    let mut buf: [u8; 8] = [0; 8];
    let n = input.read(&mut buf).expect("couldn't read u64");
    if n < 8 {
        println!("n = {}", n);
        panic!("couldn't read u64");
    }
    let tmp: u64 = unsafe {
        transmute(buf)
    };
    tmp.to_be()
}

fn read_one_table_record(input: &mut Read) -> Tree {
    let value = read_one_byte(input);
    let mut buf: [u8; 8] = [0; 8];
    let error_message = "couldn't read table record";
    let n = input.read(&mut buf).expect(error_message);
    if n < 8 {
        println!("n = {}", n);
        panic!(error_message);
    }

    let mut count: u64 = unsafe {
        transmute(buf)
    };
    count = count.to_be();

    Leaf {
        count,
        value
    }
}

fn read_one_byte(input: &mut Read) -> u8 {
    let mut buf: [u8; 1] = [0; 1];
    let n = input.read(&mut buf).expect("Error reading from input");
    if n < 1 {
        panic!("could not read a byte");
    }
    buf[0]
}

pub fn compress() {
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
    let mut output = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open("test.txt.hc")
        .expect("could open output file");

    // write to output file the length of the list of byte occurences
    let leaves = make_tree_leaves(list);
    let list_len: u8 = leaves.len() as u8;
    let list_len_buf: [u8; 1] = [list_len; 1];
    output.write_all(&list_len_buf);
    // write to output file the list of byte occurences
    let leaves_tmp = make_tree_leaves(list);
    for l in leaves_tmp {
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
            _ => {
                panic!("this should never happen")
            }
        }
    }

    // write the size of the original file (number of bytes)
    let mut size: u64 = 0;
    for l in leaves {
        match l {
            Leaf {count, value} => size = size + count,
            _ => panic!("this should never happen")
        }
    }
    let mut buf: [u8; 8] = unsafe {
        transmute(size.to_be())
    };
    output.write_all(&buf);

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