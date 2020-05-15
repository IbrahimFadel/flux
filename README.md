# Sandscript

Ok, here is my attempt at a compiled programming language. I'm gonna be doing lexing and parsing by hand. Gonna generate code with LLVM.

Here's how it works:

file.ss

```ts
const words: array<string> = ["Apple", "Car", "Toronto", "John"];
const theMeaningOfLife: int = 42;

let x: float = 1.0;

fn increaseFloat: float = (num: float) -> {
  num += 0.1;
  return num;
};

x = increaseFloat(x);

fn printHello: void = () -> {
  print("Hello!");
}

for(let i: int = 0; i < 10; i++) {
  printHello();
}
```

This gets tokenized by `src/lexer.cpp`

Next, the tokens are parsed into an AST with `src/parser.cpp`

The AST gets generated into LLVM IR (basically an assembly language) in `src/code_generation.cpp`.

I haven't done much code generation yet, as i'm still working on parsing. Once i finish parsing, i'll figure out generating the LLVM IR which should hopefully be the most difficult step. After that, turn LLVM IR into a binary.
