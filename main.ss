object Person {
    i32 age;
    i16 address;
};

fn get_person(i32 x, i16 y) -> Person {
    Person new_person = {
        age = x;
        address = y;
    };

    return new_person;
};

fn main() -> Person {
    Person ibrahim = get_person(16, 123);
    return ibrahim;
};