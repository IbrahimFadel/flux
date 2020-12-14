fn increaseNumber(i32 num, i32 increaseBy) -> i32 {
    return num + increaseBy;
};

fn main() -> i32 {
    string formatString = "My age in %d years will be: %d";
    i32 currentAge = 16;
    i32 yearsIntoTheFuture = 10;
    i32 age = increaseNumber(currentAge, yearsIntoTheFuture);

    printf(formatString, yearsIntoTheFuture, age);

    return 0;
};