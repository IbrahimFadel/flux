fn sum(i32 num1, i32 num2) -> i8 {
    i32 sum = num1 + num2;
    i8 tst = toi8(sum);
    return tst;
}

fn main() -> i8 {
    i32 x = 10;
    i32 y = 4;

    i8 res = sum(x, y);

    return res;
}