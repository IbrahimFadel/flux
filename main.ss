fn main() -> i32 {
    i32 x = 6;
    i32 *z;

    z = &x;

    i32 z_val = *z;

    i32 final = z_val;

    if(final == 5) {
        return 2;
    }

    return final;
};