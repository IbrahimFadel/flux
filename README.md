# PI

A statically typed language made with LLVM

Example program:

```
/*
 * Classes have just been added so not everything works yet. Ex. 'pub' is being parsed but doesn't mean anything yet (for both properties and methods)
/*
class Person {
    pub mut i32 age;
    mut i16 address;
    mut u8 test = 5; // Default value
    mut u64 hi = 5;

    constructor(i32 age, i16 address, u8 test, u64 hi) {
        this->age = age;
        this->address = address;
        this->test = test;
        this->hi = hi;
    }

    pub fn print() -> void {

    }

    fn testFunction(i8 param) -> u8 {
        return 0;
    }
}

// only 'y' can be modified, and remember to typecast when adding!
fn add(i32 x, mut i8 y) -> i32 {
  y = 5;
  return x + i32(y);
}

// entry function
pub fn main() -> u32 {
    Person *johnDoe = new Person(16, 153, 0, 10);

    mut u32 x = 0;

    /*
     * '&&' has greater precedence than '||'
     * so this is (x<10 or x==10) && x == 1
    */
    if(x < 10 || x == 10 && x == 0) {
        x = 20;
    }
    
    i32 result = add(x, 13);

    return result + i32(johnDoe->age);
}
```

This project had undergone many rewrites, so some functionality is removed and re-added every now and again. For instance, imports used to be a thing, but i haven't reimplemented them yet.

## Goals

The goal of this language is to enforce good coding practices. Variables and class properties are private and constant by default, and parameters are immutable by default. I've tried to design it in a way that nothing goes on 'behind the scenes' ie. the code does exactly what you tell it to. Want to add two numbers of different types? The user has to perform a typecast first.

I also wanted to make the type system simple and easy by using short names (i always hated writing out `uintX_t`). Here are all the primitive types in Pi:

| Type | Description         |
|------|---------------------|
| i64  | Signed 64 bit int   |
| u64  | Unsigned 64 bit int |
| i32  | Signed 32 bit int   |
| u32  | Unsigned 32 bit int |
| i16  | Signed 16 bit int   |
| u16  | Unsigned 16 bit int |
| i8   | Signed 8 bit int    |
| u8   | Unsigned 8 bit int  |
| bool | Boolean, 1 bit int  |
| f64  | 64 bit float        |
| f32  | 32 bit float        |

I will be implementing garbage collection as soon as possible, but I'm really focused on getting everything with functions, variables, and classes working 100% correctly without any bugs. Then I can start doing the intersting things: optimizations, static/dynamic analysis for things like memory leaks and undefined behaviour (maybe).

I want to make sure that the user never has to worry about declaring a function with the same name as some function built into the language. For this reason, and functions built into pi will be prefixed with a `@`. For example, a user could declare `fn malloc(i32 size) -> i8* {}` and call `i8* res = malloc(16);` which would not be confused for `i8* res = @malloc(16);`.

## Usage

Compile the project with cmake, then invoke the pi compiler:

```bash
# pi warns about everything by default
# In this example, set warnings to errors, but don't warn about unnecessary typecasts
pi inputfile.pi otherinputfile.pi -Werror -Wno-unnecessary-typecast -o main
```
