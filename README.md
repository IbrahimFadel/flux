# yabl
Yet Another Bad (Programming) Language

This is my attempt at making a programming language.
Written in c++ because why not.
Hopefully it will be compiled, but that's probably not gonna happen - So i'll develop it as an interpreted language first.
Basically, i wan't to try to make something atleast somewhat functional, that's the goal.
~~Currently the lexer works~~ <-- i broke it again

## Example yabl file

```
int i = 0;
while(5 < 6) {
  print("5 will always be less than six"); // <-- the 5 isn't seen as a number because it's in a string and this is a comment
  i = i + 1; // no syntactical sugar
}
```
