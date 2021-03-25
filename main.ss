fn test(i32 x, i16 y) -> u32 {
    return x + i32(y);
}

pub fn main() -> u32 {

    mut i32 x = 0;

    for(mut i32 i = 0; i < 20; i = i + 1) {
        x = i;
    }

    u32 res = test(5, 10);

    u32 final = res + u32(x);

    return u32(x);
}