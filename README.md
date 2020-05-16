# Sandscript

Ok, here is my attempt at a compiled programming language. I'm gonna be doing lexing and parsing by hand. Gonna generate code with LLVM.

Here's how it works:

file.ss

```ts
const x: float = 3 * 1.0 * 3 + 5 * 9 * 3;

fn sum: float = () -> {
  return(6.0);
}
```

This gets tokenized by `src/lexer.cpp`

```
['const' : 0] - 0 0
['x' : 1] - 0 6
[':' : 2] - 0 7
['float' : 1] - 0 9
['=' : 2] - 0 15
['3' : 3] - 0 17
['*' : 2] - 0 19
['1.0' : 3] - 0 21
['*' : 2] - 0 25
['3' : 3] - 0 27
['+' : 2] - 0 29
['5' : 3] - 0 31
['*' : 2] - 0 33
['9' : 3] - 0 35
['*' : 2] - 0 37
['3' : 3] - 0 39
[';' : 5] - 0 40
['fn' : 0] - 2 0
['sum' : 1] - 2 3
[':' : 2] - 2 6
['float' : 1] - 2 8
['=' : 2] - 2 14
['(' : 4] - 2 16
[')' : 4] - 2 17
['->' : 1] - 2 19
['{' : 4] - 2 22
['return' : 0] - 3 2
['(' : 4] - 3 8
['6.0' : 3] - 3 9
[')' : 4] - 3 12
[';' : 5] - 3 13
['}' : 4] - 4 0
```

Next, the tokens are parsed into an AST with `src/parser.cpp`

```
Constant Declaration Node: x
Function Declaration: sum
PARAMS:
Return: 3
```

The AST gets generated into LLVM IR (basically an assembly language) in `src/code_generation.cpp`.

```
float 1.440000e+02
define double @sum() {
entry:
  ret float 6.000000e+00
}
```

Parsing and Code generation are both sort of being developed together. I'll add a new feature in parsing, then implement code generation for it.

I don't think i'll touch compiling the LLVM IR into binary for a while, cause it shouldn't be much of an issue? I don't really know. I might experiment with it soon.
