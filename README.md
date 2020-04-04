# Sandscript

Sandscript is a simple declarative programming language

Using LLVM for code generation

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
