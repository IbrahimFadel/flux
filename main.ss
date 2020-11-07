i8 globvar = 10 + 5;
i32 othernum = 25;

fn sum(i32 num1, i32 num2) -> i8 {
    return toi8(num1) + toi8(num2);
}

fn main() -> i8 {
    i8 mysum = sum(20, 10);
    i8 final = toi8(othernum) + globvar * mysum;
    return final;
}