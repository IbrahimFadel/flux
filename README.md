# Sandscript

Sandscript is a small compiled, statically typed language. It's not very useful, but it's been fun working on.

Rewriting in yet again -- and hey, there's new syntax:

```cpp
fn increaseNumber(i32 num, i32 increaseBy) -> i32 {
    return num + increaseBy;
};

fn main() -> i32 {
    string formatString = "My age in %d years will be: %d";
    i32 currentAge = 16;
    i32 yearsIntoTheFuture = 10;
    i32 age = increaseNumber(currentAge, yearsIntoTheFuture);

    printf(formatString, yearsIntoTheFuture, age);

    return 0;
};
```

Primitive types:

```
| Type Description      | Sandscript Type Name |
|-----------------------|----------------------|
| Signed 64 bit integer | i64                  |
| Signed 32 bit integer | i32                  |
| Signed 16 bit integer | i16                  |
| Signed 8 bit integer  | i8                   |
| Unsigned 1 bit integer| bool                 |
| Array of bytes        | string               |
| Array of any          | array<type>          |
```

User defined types:
```
object Person {
    i32 age;
    string name;
};

Person ibrahimFadel = {
    age = 16;
    name = "Ibrahim Fadel";
};
```

Sandscript also has basic syntax for c-like pointer/reference types:
```
i32 x = 0;
i32 *y = &x;
i32 z = *y;
```

Strings are literally just arrays of ```i8```s right now because that was a simple implementation, but in the future, they will be an object with methods like ```length()```. Although the main reason the ```string``` type is useful to me right now is because I haven't really done anything with the array type -- it's pretty much useless right now because you can't reassign values or get values.

Printing will likeley change -- I don't want the c ```printf``` function exposed to the user like that by default. All functions that are a part of the standard sandscript library like ```print``` should be included and called with a ```@``` prefix. The aim of that is that users will never have to be worried about defining a function that is reserved, ie. they can define a function called ```print``` which is entirely different from a standard library ```@print()``` call. I think I'll just make a sandscript file and have that link in the final compile stages with whatever the user has made.

Overall, this entire project feels incomplete/messy to me because I've been learning how it all works as I implement things. This makes me really feel that I might rewrite yet again.