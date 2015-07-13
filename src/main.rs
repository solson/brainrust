use std::env;
use std::fs::File;
use std::io::{self, Read, Write, stdin, stdout};
use std::iter::repeat;
use std::path::Path;

// A brainfuck instruction.
enum Op {
    Inc,   // +
    Dec,   // -
    Left,  // <
    Right, // >
    Read,  // ,
    Write, // .

    // Each loop instruction stores the index of its matching loop instruction.
    LoopStart(usize), // [
    LoopEnd(usize),   // ]
}

// Parse errors contain the index of the offending character in the original program source.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ParseError { UnmatchedLoopStart(usize), UnmatchedLoopEnd(usize) }

fn parse(program: &str) -> Result<Vec<Op>, ParseError> {
    let mut ops = Vec::new();
    let mut loop_stack = Vec::new();

    for (i, op) in program.chars().enumerate() {
        match op {
            '+' => ops.push(Op::Inc),
            '-' => ops.push(Op::Dec),
            '<' => ops.push(Op::Left),
            '>' => ops.push(Op::Right),
            ',' => ops.push(Op::Read),
            '.' => ops.push(Op::Write),
            '[' => {
                loop_stack.push(i);
                ops.push(Op::LoopStart(0));
            },
            ']' => match loop_stack.pop() {
                Some(loop_start) => ops.push(Op::LoopEnd(loop_start)),
                None             => return Err(ParseError::UnmatchedLoopEnd(i)),
            },
            _   => {}
        }
    }

    if loop_stack.is_empty() {
        Ok(ops)
    } else {
        Err(ParseError::UnmatchedLoopStart(loop_stack[0]))
    }
}

trait Tape {
    fn go_left(&mut self);
    fn go_right(&mut self);
    fn inc(&mut self);
    fn dec(&mut self);
    fn read(&self) -> u8;
    fn write(&mut self, byte: u8);
}

struct SimpleTape {
    pos: usize,
    data: Vec<u8>,
}

impl SimpleTape {
    fn new(size: usize) -> SimpleTape {
        SimpleTape { pos: 0, data: repeat(0u8).take(size).collect() }
    }
}

impl Tape for SimpleTape {
    fn go_left(&mut self)  { self.pos -= 1; }
    fn go_right(&mut self) { self.pos += 1; }
    fn inc(&mut self) { self.data[self.pos] += 1; }
    fn dec(&mut self) { self.data[self.pos] -= 1; }
    fn read(&self) -> u8 { self.data[self.pos] }
    fn write(&mut self, byte: u8) { self.data[self.pos] = byte; }
}

struct CircularTape {
    pos: usize,
    data: Vec<u8>,
}

impl CircularTape {
    fn new(size: usize) -> CircularTape {
        CircularTape { pos: 0, data: repeat(0u8).take(size).collect() }
    }
}

impl Tape for CircularTape {
    fn go_left(&mut self)  { self.pos = (self.pos - 1) % self.data.len(); }
    fn go_right(&mut self) { self.pos = (self.pos + 1) % self.data.len(); }
    fn inc(&mut self) { self.data[self.pos] += 1; }
    fn dec(&mut self) { self.data[self.pos] -= 1; }
    fn read(&self) -> u8 { self.data[self.pos] }
    fn write(&mut self, byte: u8) { self.data[self.pos] = byte; }
}

fn execute<R: Read, W: Write, T: Tape>(program: Vec<Op>, input: &mut R, output: &mut W,
                                          tape: &mut T) -> io::Result<()> {
    let mut ip: usize = 0; // Instruction pointer.

    while ip < program.len() {
        match program[ip] {
            Op::Inc   => tape.inc(),
            Op::Dec   => tape.dec(),
            Op::Left  => tape.go_left(),
            Op::Right => tape.go_right(),
            Op::Read  => {
                let mut byte = [0u8; 1];
                match input.read(&mut byte) {
                    Ok(_)  => tape.write(byte[0]),
                    Err(_) => {} // Do nothing on EOF.
                }
            },
            Op::Write => { try!(output.write(&[tape.read(); 1])); },
            Op::LoopStart(loop_end) => if tape.read() == 0 { ip = loop_end; },
            Op::LoopEnd(loop_start) => if tape.read() != 0 { ip = loop_start; },
        }

        ip += 1;
    }

    Ok(())
}

fn read_file(name: &str) -> io::Result<String> {
    File::open(&Path::new(name)).and_then(|mut file| {
        let mut s = String::new();
        try!(file.read_to_string(&mut s));
        Ok(s)
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: {} <file>", args[0]);
        return;
    }

    let mut tape = SimpleTape::new(1024);

    read_file(&args[1]).and_then(|program| {
        execute(parse(&program).unwrap(), &mut stdin(), &mut stdout(), &mut tape)
    }).unwrap();
}

#[test]
fn hello_world() {
    use std::io::util::NullReader;

    let program = include_str!("../hello_world.bf");
    let mut output = Vec::new();
    let mut tape = SimpleTape::new(1024);
    execute(parse(program).unwrap(), &mut NullReader, &mut output, &mut tape).unwrap();
    assert_eq!(output.as_slice(), b"Hello World!\n");
}
