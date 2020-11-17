fn main() -> i32 
{
    object my_object {
        i32 fav_number;
        i16 address;
    };

    my_object test = {
        fav_number = 10;
        address = 5;
    };

    return 0;
}