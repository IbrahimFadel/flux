struct string {
    i8 *buffer;
    i64 length;
    i64 maxLength;
    i64 factor;
};

fn main() -> void {
    i64 len = 0;
    len = len + 1;

    string test;
    
    test->length = test->maxLength;
}