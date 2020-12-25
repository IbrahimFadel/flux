fn main(i32 **param1, i32 param2) -> i32 {
    object Person {
        i32 **x;
        i32 y;
    };


    i32 x = 5;
    i32 y = &x;
    i32 z = &y;

    Person ibrahim;
    ibrahim.x = z;
    ibrahim.y = *(*z);
}