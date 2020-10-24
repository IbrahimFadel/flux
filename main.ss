fn sum(i32 num1, i32 num2) -> i8 {
    i32 res = num1 + num2;
    return toi8(res);
}

fn main() -> i8 {
    i8 thesum = sum(5, 3);

    return thesum;
}