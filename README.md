# huffman_code

I saw a [video about huffman codes](https://www.youtube.com/watch?v=JsTptu56GM8) and I made a program based on what I saw.

# Get Started


1) make sure you have [rust and cargo](https://www.rust-lang.org/en-US/install.html) installed
2) clone this repo
3) cd into the project directory
4) create a file called test.txt and put some text into it.
```bash
cat 'hello world this is my text' > test.txt
```

5) run the program
```bash
cargo run
```
6) The program should have created a new file test.txt.hc that contains the compressed data.
7) edit [main.rs](https://github.com/murphym18/huffman_code/blob/master/src/main.rs#L4) comment line 5 and uncomment line 6
8) run the program again
``` bash
cargo run
```
9) the program should have created a new file tmp.txt that contains the same data the original file (test.txt)
10) bonus: check that the files are the same
```bash
diff test.txt tmp.txt
```