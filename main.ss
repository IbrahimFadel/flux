struct Person {
    i32 age;
    i16 address;
};

fn test(Person p) -> Person * {
    return &p;
}

fn main() -> void {
    Person ibrahim = {
        age: 36;
        address: 5;
    };

    Person *p = &ibrahim;
    Person *new = test(ibrahim);
}