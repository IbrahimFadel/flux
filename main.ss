class Person {
    pub mut i32 age;
    mut i16 address;
    mut u8 test = 5;
    mut u64 hi = 5;

    constructor(i32 age, i16 address, u8 test, u64 hi) {
        this->age = age;
        this->address = address;
        this->test = test;
        this->hi = hi;
    }

    pub fn print() -> void {

    }

    fn testFunction(i8 param) -> u8 {
        return 0;
    }
}

fn main() -> i32 {
    Person *person = new Person(16, 5, 0, 0);

    return 0;
}