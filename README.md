# PI

A statically typed language made with LLVM

Example sandscript file:

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

## Usage

Compile the project with cmake, then use the ssc compiler:

```bash
# pi warns about everything by default
# In this example, set warnings to errors, but don't warn about unnecessary typecasts
pi inputfile.pi otherinputfile.pi -Werror -Wno-unnecessary-typecast -o main
```
