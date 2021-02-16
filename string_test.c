#include <stdlib.h>
#include <stdint.h>
#include <stdio.h>

//? This is a file that should be compiled with `std/string.ss` to test it out

typedef struct test
{
    char *buffer;
    int64_t length;
    int64_t maxLength;
    int64_t factor;
} test;

int main()
{
    test *foo = malloc(sizeof(test));

    string_create_default(foo);
    string_add_char(foo, 72);
    printf("%c", *foo->buffer);
    string_add_char(foo, 101);
    printf("%c", *foo->buffer);
    string_add_char(foo, 108);
    printf("%c", *foo->buffer);
    string_add_char(foo, 108);
    printf("%c", *foo->buffer);
    string_add_char(foo, 111);
    printf("%c", *foo->buffer);
    string_add_char(foo, 33);
    printf("%c", *foo->buffer);
    string_delete(foo);

    return 0;
}