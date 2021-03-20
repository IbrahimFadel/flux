# Sandscript

A statically typed language made with LLVM

Example sandscript file:

```
// only 'y' can be modified, and remember to typecast when adding!
fn add(i32 x, mut i8 y) -> i32 {
  y = 5;
  return x + i32(y);
}

// entry function
pub fn main() -> u32 {
    mut u32 x = 0;

    /*
     * '&&' has greater precedence than '||'
     * so this is (x<10 or x==10) && x == 1
    */
    if(x < 10 || x == 10 && x == 0) {
        x = 20;
    }
    
    i32 result = add(x, 13);

    return result;
}
```
