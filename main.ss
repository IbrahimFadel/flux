i8 globvar = 10 + 5;
i32 othernum = 25;

fn sum(i32 num1, i32 num2) -> i8 {
    i8 retval = toi8(num1) + toi8(num2);
    return retval;
}

fn main() -> i8 {
    i32 x = 20;
    i32 y = 30;
    i8 mysum = sum(x, y);
    i8 final = toi8(othernum) + globvar * mysum;
    return final;
}