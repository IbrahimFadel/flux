# yabl

Yet Another Bad (Programming) Language

This is my attempt at making a programming language.
Written in c++ because why not.
Hopefully it will be compiled, but that's probably not gonna happen - So i'll develop it as an interpreted language first.

Basically, i wan't to try to make something atleast somewhat functional, that's the goal.

## Example yabl file

```cpp
let x = 0;
let word = "rick astley";
let other_word = "gonna give you up";

while(x < 100) {
  print(x);
  x = x + 1;
}

if(word != other_word) {
  print(word, "is never", other_word);
}
```

## Goals

- Optional entry/exit functions

```cpp
#ENTRY my_entry_fn
#EXIT my_exit_fn
```

- Error/Warning logs

```python
error("My error message");
warning("My warning message");
```

- Easy input

```python
let in = input("Enter input: ");
```

- String formatting

```
let time = "afternoon";
let name = "Rick Astley";
let phrase = f"Good {time}, {name}!";
```

- Maybe?? A project manager sort of thing - like cargo(rust-lang)

```bash
yabl new hello_word_project
```

## Install

```bash
git clone https://github.com/IbrahimFadel/yabl.git
```

```bash
cd yabl
```

```bash
mkdir -p build/parts && make && make install
```

## Usage

```
yabl file.ybl
```

## Progress

This example will be using the following as the input file:

```cpp
let x = 0;
while(x < 50) {
  if(x == 10) {
    print(x, "<-- x is 10");
  }
  x = x + 1;
}
```

### Lexer

Currently, the lexer is pretty simple. It takes an array of lines, which is an array of characters, and breaks them up into tokens with line numbers, positions, and types/values.

```
[ 1: 'let' ] - ln:0 pos:0
[ 0: 'x' ] - ln:0 pos:4
[ 0: '=' ] - ln:0 pos:6
[ 3: '0' ] - ln:0 pos:8
[ 5: ';' ] - ln:0 pos:9
[ 1: 'while' ] - ln:1 pos:0
[ 4: '(' ] - ln:1 pos:5
[ 0: 'x' ] - ln:1 pos:6
[ 2: '<' ] - ln:1 pos:8
[ 3: '50' ] - ln:1 pos:10
[ 4: ')' ] - ln:1 pos:12
[ 4: '{' ] - ln:1 pos:14
[ 1: 'if' ] - ln:2 pos:2
[ 4: '(' ] - ln:2 pos:4
[ 0: 'x' ] - ln:2 pos:5
[ 2: '==' ] - ln:2 pos:8
[ 3: '10' ] - ln:2 pos:10
[ 4: ')' ] - ln:2 pos:12
[ 4: '{' ] - ln:2 pos:14
[ 1: 'print' ] - ln:3 pos:4
[ 4: '(' ] - ln:3 pos:9
[ 0: 'x' ] - ln:3 pos:10
[ 4: ',' ] - ln:3 pos:10
[ 3: '"<-- x is 10"' ] - ln:3 pos:13
[ 4: ')' ] - ln:3 pos:26
[ 5: ';' ] - ln:3 pos:27
[ 4: '}' ] - ln:4 pos:2
[ 0: 'x' ] - ln:5 pos:2
[ 0: '=' ] - ln:5 pos:4
[ 0: 'x' ] - ln:5 pos:6
[ 2: '+' ] - ln:5 pos:8
[ 3: '1' ] - ln:5 pos:10
[ 5: ';' ] - ln:5 pos:11
[ 4: '}' ] - ln:6 pos:0
```

### Parser

The parser then takes these tokens and builds a parse tree / ast

```
------ NODE ------
LET: x = 0

------ END NODE ------
------ NODE ------
WHILE LOOP:
CONDITION: x < 50
THEN:
IF STATEMENT:
CONDITION: x == 10
THEN:
FUNCTION_CALL: print
-- PARAMETERS --
IDENTIFIER: x

LITERAL: "<-- x is 10"

-- END PARAMETERS --

SEPERATOR: )

EOL
SEPERATOR: }

-- END THEN --

ASSIGN: x
EOL
SEPERATOR: }

-- END THEN --

------ END NODE ------
```

### Interpereter

Here are the nodes that are currently being handled:

```cpp
switch (node.type)
{
  case Node_Types::function_call:
    if (node.function_call_name == "print")
    {
      _print(node);
    }
    break;
  case Node_Types::_while:
    Interpreter::_while(node);
    break;
  case Node_Types::_if:
    Interpreter::_if(node);
    break;
  case Node_Types::let:
    Interpreter::let(node);
    break;
  case Node_Types::assign:
    Interpreter::assign(node);
    break;
  default:
    break;
}
```
