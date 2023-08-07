# chi8disasm

A chip-8 disassembler command line utility written in rust

## Usage
By default, chi8disasm will read a stream of bytes from stdin and write a long string seperated by newlines to stdout
For convenience, the `-i` or `--input` argument will make it read from a file instead of stdin and the `-o` or `--output` argument will make it write to a file instead of stdout


Ex. 

Both of the following will read the bytes from pong.ch8, disassemble them into chip8 assembly, and write the result to pong.asm
```
cat pong.ch8 | ch8disasm > pong.asm
```
```
ch8disasm -i pong.ch8 -o pong.asm
```

## Rationale
Obviously lots of utilities like this already exist, but this was an opportunity for me to apply some basic rust skills to a task that isn't totally trivial 
(though, admittedly, it is pretty simple). The structure is based closely on the minigrep project from the rust book. Writing it reinforced a lot what I learned
from the book but also taught me more about error handling, the `From` trait, `std::io` and `std::fs`. I'd like to rewrite my chip8 emulator in rust in the near
future, so I made sure to structure this project so it can be used in a library in that when I get around to it, so I can use it to implement the debug mode I
put into my python chip8 emulator. 

