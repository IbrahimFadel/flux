fn main() -> i32 {
    i32 x = 5;
    if(x == 5) {
        i32 y = 5;

        if(x == y) {
            return 5;
        }

        return 2;
    }

    return 0;
};