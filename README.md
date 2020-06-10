# Sandscript

Ok, here is my attempt at a compiled programming language. I'm gonna be doing lexing and parsing by hand. Gonna generate code with LLVM.

Here's how it works:

file.ss

```ts
fn main: int = (number1: int, number2: float) -> {
  let sum: float = 1 * 3 + 5;
  let x: float = sum + 2;
  return(5);
}
```

This gets tokenized by `src/lexer.cpp`

Format 
[token : Token_Type] - line_number line_position

```
['fn' : 0] - 0 0
['main' : 1] - 0 3
[':' : 2] - 0 7
['int' : 0] - 0 9
['=' : 2] - 0 13
['(' : 4] - 0 15
['number1' : 1] - 0 16
[':' : 2] - 0 23
['int' : 0] - 0 25
[',' : 5] - 0 28
['number2' : 1] - 0 30
[':' : 2] - 0 37
['float' : 1] - 0 39
[')' : 4] - 0 44
['->' : 1] - 0 46
['{' : 4] - 0 49
['let' : 0] - 1 2
['sum' : 1] - 1 6
[':' : 2] - 1 9
['float' : 1] - 1 11
['=' : 2] - 1 17
['1' : 3] - 1 19
['*' : 2] - 1 21
['3' : 3] - 1 23
['+' : 2] - 1 25
['5' : 3] - 1 27
[';' : 5] - 1 28
['let' : 0] - 2 2
['x' : 1] - 2 6
[':' : 2] - 2 7
['float' : 1] - 2 9
['=' : 2] - 2 15
['sum' : 1] - 2 17
['+' : 2] - 2 21
['2' : 3] - 2 23
[';' : 5] - 2 24
['return' : 0] - 3 2
['(' : 4] - 3 8
['5' : 3] - 3 9
[')' : 4] - 3 10
[';' : 5] - 3 11
['}' : 4] - 4 0
```

Next, the tokens are parsed into an AST with `src/parser.cpp`

```
Function Declaration: main
PARAMS: 
number1: 0
number2: 3
Return Type: 0
Then: 
Variable Declaration Node: sum
Variable Declaration Node: x
```

The AST gets generated into LLVM IR (basically an assembly language) in `src/code_generation.cpp`.

```
define i32 @main(i32 %number1, float %number2) {
entry:
  %sum = alloca float
  %sum1 = alloca float
  %sum2 = alloca float
  store float 3.000000e+00, float* %sum2
  %sum3 = load float, float* %sum2
  %sum4 = alloca float
  store float 5.000000e+00, float* %sum4
  %sum5 = load float, float* %sum4
  %sum6 = alloca float
  %sum7 = fadd float %sum3, %sum5
  store float %sum7, float* %sum6
  %sum8 = load float, float* %sum6
  store float %sum8, float* %sum
  %sum_loaded = load float, float* %sum
  %x = alloca float
  %x9 = alloca float
  store float %sum_loaded, float* %x9
  %x10 = load float, float* %x9
  %x11 = alloca float
  store float 2.000000e+00, float* %x11
  %x12 = load float, float* %x11
  %x13 = alloca float
  %x14 = fadd float %x10, %x12
  store float %x14, float* %x13
  %x15 = load float, float* %x13
  store float %x15, float* %x
  %x_loaded = load float, float* %x
  ret i32 5
}
```

After optimization:

```
define i32 @main(i32 %number1, float %number2) {
entry:
  ret i32 5
}
```

Parsing and Code generation are both sort of being developed together. I'll add a new feature in parsing, then implement code generation for it.

I don't think i'll touch compiling the LLVM IR into binary for a while, cause it shouldn't be much of an issue? I don't really know. I might experiment with it soon.
