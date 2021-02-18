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
    string_add_char(foo, 'H');
    string_add_char(foo, 'e');
    string_add_char(foo, 'l');
    string_add_char(foo, 'l');
    string_add_char(foo, 'o');
    string_add_char(foo, '!');

    printf("Buffer: %s\n", foo->buffer);
    printf("Length: %ld\n", foo->length);
    printf("Max Length: %ld\n", foo->maxLength);
    printf("Factor: %ld\n", foo->factor);

    string_delete(foo);

    return 0;
}