# yabl

Yet Another Bad (Programming) Language

This is my attempt at making a programming language.
Written in c++ because why not.
Hopefully it will be compiled, but that's probably not gonna happen - So i'll develop it as an interpreted language first.

Basically, i wan't to try to make something atleast somewhat functional, that's the goal.

### I've decided to try to get something functional, then take what i've learnt and create a new language that's more unique

### My own syntax and stuff. For now i', gonna keep it more C like

## Example yabl file

```cpp

let my_fn = (a, b) -> {
  return a + b;
}

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

Run a file

```
yabl file.ybl
```

Output the tokens, and parse tree before hand (verbose output)

```
yabl file.ybl -v
```

Create new project (not sure if this is gonna stay... feels a bit useless)

```
yabl new my_project_name
```

## Progress

This example will be using the following as the input file:

```cpp
let x = 1;

while(x < 101) {
  if(x % 5 == 0 && x % 3 == 0) {
    print("Fizz buzz:", x);
  } else if(x % 3 == 0) {
    print("Fizz:", x);
  } else if(x % 3 == 0) {
    print("Buzz:", x);
  } else {
    print(x);
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
[ 3: '1' ] - ln:0 pos:8
[ 5: ';' ] - ln:0 pos:9
[ 1: 'while' ] - ln:2 pos:0
[ 4: '(' ] - ln:2 pos:5
[ 0: 'x' ] - ln:2 pos:6
[ 2: '<' ] - ln:2 pos:8
[ 3: '101' ] - ln:2 pos:10
[ 4: ')' ] - ln:2 pos:13
[ 4: '{' ] - ln:2 pos:15
[ 1: 'if' ] - ln:3 pos:2
[ 4: '(' ] - ln:3 pos:4
[ 0: 'x' ] - ln:3 pos:5
[ 2: '%' ] - ln:3 pos:7
[ 3: '5' ] - ln:3 pos:9
[ 2: '==' ] - ln:3 pos:12
[ 3: '0' ] - ln:3 pos:14
[ 2: '&&' ] - ln:3 pos:17
[ 0: 'x' ] - ln:3 pos:19
[ 2: '%' ] - ln:3 pos:21
[ 3: '3' ] - ln:3 pos:23
[ 2: '==' ] - ln:3 pos:26
[ 3: '0' ] - ln:3 pos:28
[ 4: ')' ] - ln:3 pos:29
[ 4: '{' ] - ln:3 pos:31
[ 1: 'print' ] - ln:4 pos:4
[ 4: '(' ] - ln:4 pos:9
[ 3: '"Fizz buzz:"' ] - ln:4 pos:10
[ 0: ',' ] - ln:4 pos:22
[ 0: 'x' ] - ln:4 pos:24
[ 4: ')' ] - ln:4 pos:24
[ 5: ';' ] - ln:4 pos:26
[ 4: '}' ] - ln:5 pos:2
[ 1: 'else' ] - ln:5 pos:4
[ 1: 'if' ] - ln:5 pos:9
[ 4: '(' ] - ln:5 pos:11
[ 0: 'x' ] - ln:5 pos:12
[ 2: '%' ] - ln:5 pos:14
[ 3: '3' ] - ln:5 pos:16
[ 2: '==' ] - ln:5 pos:19
[ 3: '0' ] - ln:5 pos:21
[ 4: ')' ] - ln:5 pos:22
[ 4: '{' ] - ln:5 pos:24
[ 1: 'print' ] - ln:6 pos:4
[ 4: '(' ] - ln:6 pos:9
[ 3: '"Fizz:"' ] - ln:6 pos:10
[ 0: ',' ] - ln:6 pos:17
[ 0: 'x' ] - ln:6 pos:19
[ 4: ')' ] - ln:6 pos:19
[ 5: ';' ] - ln:6 pos:21
[ 4: '}' ] - ln:7 pos:2
[ 1: 'else' ] - ln:7 pos:4
[ 1: 'if' ] - ln:7 pos:9
[ 4: '(' ] - ln:7 pos:11
[ 0: 'x' ] - ln:7 pos:12
[ 2: '%' ] - ln:7 pos:14
[ 3: '3' ] - ln:7 pos:16
[ 2: '==' ] - ln:7 pos:19
[ 3: '0' ] - ln:7 pos:21
[ 4: ')' ] - ln:7 pos:22
[ 4: '{' ] - ln:7 pos:24
[ 1: 'print' ] - ln:8 pos:4
[ 4: '(' ] - ln:8 pos:9
[ 3: '"Buzz:"' ] - ln:8 pos:10
[ 0: ',' ] - ln:8 pos:17
[ 0: 'x' ] - ln:8 pos:19
[ 4: ')' ] - ln:8 pos:19
[ 5: ';' ] - ln:8 pos:21
[ 4: '}' ] - ln:9 pos:2
[ 1: 'else' ] - ln:9 pos:4
[ 4: '{' ] - ln:9 pos:9
[ 1: 'print' ] - ln:10 pos:4
[ 4: '(' ] - ln:10 pos:9
[ 0: 'x' ] - ln:10 pos:10
[ 4: ')' ] - ln:10 pos:10
[ 5: ';' ] - ln:10 pos:12
[ 4: '}' ] - ln:11 pos:2
[ 0: 'x' ] - ln:12 pos:2
[ 0: '=' ] - ln:12 pos:4
[ 0: 'x' ] - ln:12 pos:6
[ 2: '+' ] - ln:12 pos:8
[ 3: '1' ] - ln:12 pos:10
[ 5: ';' ] - ln:12 pos:11
[ 4: '}' ] - ln:13 pos:0
```

### Parser

The parser then takes these tokens and builds a parse tree / ast

```
------ NODE ------
LET: x = 1

------ END NODE ------
------ NODE ------
WHILE LOOP:
CONDITION(s):
x < 101
THEN:
SEPERATOR: {

IF STATEMENT:
CONDITION(s):
x % 5
x % 3
THEN:
SEPERATOR: {

FUNCTION_CALL: print
-- PARAMETERS --
LITERAL: "Fizz buzz:"

IDENTIFIER: x

-- END PARAMETERS --

EOL
SEPERATOR: }

-- END THEN --

ELSE IF:
CONDITION(s):
x % 3
THEN:
SEPERATOR: {
FUNCTION_CALL: print
-- PARAMETERS --
LITERAL: "Fizz:"

IDENTIFIER: x

-- END PARAMETERS --
EOLSEPERATOR: }

ELSE IF:
CONDITION(s):
x % 3
THEN:
SEPERATOR: {
FUNCTION_CALL: print
-- PARAMETERS --
LITERAL: "Buzz:"

IDENTIFIER: x

-- END PARAMETERS --
EOLSEPERATOR: }

ELSE:
SEPERATOR: {
FUNCTION_CALL: print
-- PARAMETERS --
IDENTIFIER: x

-- END PARAMETERS --
SEPERATOR: }

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
    Interpreter::_while(node, parent);
    break;
  case Node_Types::_if:
    Interpreter::_if(node, parent);
    break;
  case Node_Types::else_if:
    Interpreter::else_if(nodes, i, parent);
    break;
  case Node_Types::_else:
    Interpreter::_else(nodes, i, parent);
    break;
  case Node_Types::let:
    Interpreter::let(node);
    break;
  case Node_Types::assign:
    Interpreter::assign(node);
    break;
  case Node_Types::_continue:
    Interpreter::_continue(nodes, i, parent);
    break;
  case Node_Types::_break:
    Interpreter::_break(nodes, i, parent);
    break;
  default:
    break;
}
```
