import "std/io.ss";
import "std/bytes.ss";

fn add(i32 num1, i32 num2) -> i32 {
    return num1 + num2;
}

fn main() -> void {
    i8 num = 33;
    i8 *ptr = &num;

    @print(ptr);
}