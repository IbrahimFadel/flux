object string {
    i8 *charPointer;
    i64 length;
    i64 maxLength;
    i64 factor;
};

fn createString(string *str) -> void {
    *str.length = 0;
    *str.maxLength = 0;
    *str.factor = 16;
};