# yabl
Yet Another Bad (Programming) Language

This is my attempt at making a programming language.
Written in c++ because why not.
Hopefully it will be compiled, but that's probably not gonna happen - So i'll develop it as an interpreted language first.
Basically, i wan't to try to make something atleast somewhat functional, that's the goal.
~~Currently the lexer works~~ ~~<-- i broke it again~~ the  lexer works.

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

### Lexer

Currently, the lexer is pretty simple. It takes an array of lines, which is an array of characters, and breaks them up into tokens with line numbers, positions, and types/values.

[ 1: 'while' ] - ln:0 pos:0
[ 4: '(' ] - ln:0 pos:5
[ 3: '5' ] - ln:0 pos:6
[ 2: '<' ] - ln:0 pos:8
[ 3: '6' ] - ln:0 pos:10
[ 4: ')' ] - ln:0 pos:11
[ 4: '{' ] - ln:0 pos:13
[ 1: 'if' ] - ln:1 pos:2
[ 4: '(' ] - ln:1 pos:4
[ 3: '2' ] - ln:1 pos:5
[ 2: '<' ] - ln:1 pos:7
[ 3: '1' ] - ln:1 pos:9
[ 4: ')' ] - ln:1 pos:10
[ 4: '{' ] - ln:1 pos:12
[ 1: 'print' ] - ln:2 pos:4
[ 4: '(' ] - ln:2 pos:9
[ 3: '"not gonna happen"' ] - ln:2 pos:10
[ 4: ')' ] - ln:2 pos:28
[ 5: ';' ] - ln:2 pos:29
[ 4: '}' ] - ln:3 pos:2
[ 1: 'print' ] - ln:4 pos:2
[ 4: '(' ] - ln:4 pos:7
[ 3: '"always gonna happen"' ] - ln:4 pos:8
[ 4: ')' ] - ln:4 pos:29
[ 5: ';' ] - ln:4 pos:30
[ 4: '}' ] - ln:5 pos:0

### Parser

The parser then takes these tokens and builds a parse tree / ast

------ NODE ------
WHILE LOOP: 
CONDITION: 5 < 6
THEN: 
IF STATEMENT: 
CONDITION: 2 < 1
THEN: 
FUNCTION_CALL: print
-- PARAMETERS --
LITERAL: "not gonna happen"

-- END PARAMETERS --

EOL
SEPERATOR: }

-- END THEN --

FUNCTION_CALL: print
-- PARAMETERS --
LITERAL: "always gonna happen"

-- END PARAMETERS --

EOL
SEPERATOR: }

-- END THEN --

------ END NODE ------

### Interpereter

TODO
