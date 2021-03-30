struct string {
    i8 *buffer;
    i64 length;
    i64 maxLength;
    i64 factor;
};

fn string_create_default(string *this) -> void {
    this->buffer = nullptr;
    this->length = 0;
    this->maxLength = 0;
    this->factor = 16;
}

fn string_delete(string *this) -> void {
    i8 *buf = this->buffer;
    if(buf != nullptr) {
        @free(buf);
    }
}

fn string_resize(string *this, i64 value) -> void {
    i8 *output = @malloc(value);

    i8 *buf = this->buffer;
    i64 len = this->length;

    @memcpy(output, buf, len);

    @free(buf);
    this->buffer = output;
}

fn string_add_char(string *this, i8 value) -> void {
    if(this->length == this->maxLength) {
        i64 newSize = this->maxLength + this->factor;
        string_resize(this, newSize);
        this->maxLength = newSize;
    }

    this->buffer[this->length] = value;
    this->length = this->length + 1;
}
