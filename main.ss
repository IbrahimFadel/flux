fn main() -> i32 {
    i32 x = 0;
    for(i32 i = 0; i < 100; i = i + 1) {
        if(x == 33) {
            return x;
        }
        x = x + 1;
    }

    return x;
};