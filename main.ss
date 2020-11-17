object my_object {
    i32 fav_number;
    i16 address;
};


fn main() -> my_object {
    my_object test = {
        fav_number = 10;
        address = 5;
    };

    return test;
};