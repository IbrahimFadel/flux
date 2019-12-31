# yabl
Yet Another Bad (Programming) Language

This is my attempt at making a programming language.
Written in c++ because why not.
Hopefully it will be compiled, but that's probably not gonna happen - So i'll develop it as an interpreted language first.

Basically, i wan't to try to make something atleast somewhat functional, that's the goal.

## Example yabl file

```
while(5 < 6) {
  if(2 < 1) {
    print("not gonna happen");
  }
  print("always gonna happen");
}
```

## Install

```
git clone https://github.com/IbrahimFadel/yabl.git
```
```
cd yabl
```
```
make && make install
```

## Usage

```
yabl file.ybl
```

## Progress

This example will be using the following as the input file:

```
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

So far, i've implimented while loops, if statements, and print statements(print string/int literals, or variables).

Variables can be reassigned
