# Sandscript

Ok, here is my attempt at a compiled programming language. I'm gonna be doing lexing and parsing by hand. Gonna generate code with LLVM.

Here's how it works:

file.ss

```ts
fn test: float = (number1: int, number2: float) -> {
  let sum: float = 1 + 2;
  return(5.0);
}
```

This gets tokenized by `src/lexer.cpp`

Format 
[token : Token_Type] - line_number line_position

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
Function Declaration: test
PARAMS:
PARAM 0: 0
PARAM 1: 3
Return Type: 3
THEN:
Variable Declaration: sum
```

The AST gets generated into LLVM IR (basically an assembly language) in `src/code_generation.cpp`.

```
define float @test(i32 %number1, float %number2) {
entry:
  %sum = alloca float
  store float 3.000000e+00, float* %sum
  ret float 5.000000e+00
}
```

Parsing and Code generation are both sort of being developed together. I'll add a new feature in parsing, then implement code generation for it.

I don't think i'll touch compiling the LLVM IR into binary for a while, cause it shouldn't be much of an issue? I don't really know. I might experiment with it soon.
