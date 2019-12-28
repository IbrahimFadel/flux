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
