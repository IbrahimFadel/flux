fn add(i32 &num1, i32 &num2) -> i32 {
    num1 = 23;
    return num1 + num2;
};

fn main() -> i32 {
    i32 x = 5;
    i32 z = 10;

    i32 sum = add(x, z);

    return x;
};