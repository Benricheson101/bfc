brainfuck compiler

## requirements
- rust/cargo
- llvm10
- libc
- a linker

## usage
```sh
$ cargo build --release
$ cargo insall --path .
$ bfc examples/hello.bf -o hello.o
# currently bfc does not contain a linker, so it must be done manually
# `ld` can also be used here instead of `gcc` but `ld` commands tend
# to be longer and more complex than just running `gcc`
$ gcc hello.o -o hello
$ ./hello
```

note: the included link script may or may not work for you. it's just the script I used while testing. See the example above for usage info
