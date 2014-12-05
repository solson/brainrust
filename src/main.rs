use std::io::{IoResult, Reader, Writer, stdin, stdout};

// A brainfuck instruction. 
enum Op {
    Inc,   // +
    Dec,   // -
    Left,  // <
    Right, // >
    Read,  // ,
    Write, // .

    // Each loop instruction stores the index of its matching loop instruction.
    LoopStart(uint), // [
    LoopEnd(uint),   // ]
}

// Parse errors contain the index of the offending character in the original program source.
#[deriving(PartialEq, Eq, Show)]
enum ParseError { UnmatchedLoopStart(uint), UnmatchedLoopEnd(uint) }

fn parse(program: &str) -> Result<Vec<Op>, ParseError> {
    let mut ops        = Vec::new();
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

fn execute<R: Reader, W: Writer>(program: Vec<Op>, reader: &mut R, writer: &mut W) -> IoResult<()> {
    let mut tape = [0u8, ..1024];
    let mut dp = 0u; // Data pointer.
    let mut ip = 0u; // Instruction pointer.

    while ip < program.len() {
        match program[ip] {
            Op::Inc   => tape[dp] += 1,
            Op::Dec   => tape[dp] -= 1,
            Op::Left  => dp -= 1,
            Op::Right => dp += 1,
            Op::Read  => tape[dp] = try!(reader.read_byte()),
            Op::Write => try!(writer.write_u8(tape[dp])),
            Op::LoopStart(loop_end) => if tape[dp] == 0 { ip = loop_end; },
            Op::LoopEnd(loop_start) => if tape[dp] != 0 { ip = loop_start; },
        }

        ip += 1;
    }

    Ok(())
}

fn main() {
    let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
    execute(parse(program).unwrap(), &mut stdin(), &mut stdout()).unwrap();
}
