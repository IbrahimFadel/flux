#include "scanner.h"

std::string Scanner::readFile(std::string path) {
  std::ifstream t(path);
  std::stringstream buffer;
  buffer << t.rdbuf();
  return buffer.str();
}

Scanner::Scanner::Scanner(std::string _src) {
  src = _src;

  offset = 0;
  ch = src[offset];
  pos.line = 1;
  pos.col = 1;
  pos.filename = "";
}

// https://stackoverflow.com/questions/2342162/stdstring-formatting-like-sprintf
template <typename... Args>
std::string Scanner::Scanner::fmt(const std::string& format, Args... args) {
  int size_s = std::snprintf(nullptr, 0, format.c_str(), args...) + 1;  // Extra space for '\0'
  if (size_s <= 0) {
    throw std::runtime_error("Error during formatting.");
  }
  auto size = static_cast<size_t>(size_s);
  auto buf = std::make_unique<char[]>(size);
  std::snprintf(buf.get(), size, format.c_str(), args...);
  return std::string(buf.get(), buf.get() + size - 1);  // We don't want the '\0' inside
}

void Scanner::Scanner::error(int offset, std::string message) {
  int maxPadding = 10;

  int startPos = offset;
  int len = maxPadding;

  if (offset - maxPadding >= 0) {
    startPos -= maxPadding;
    len += maxPadding + 1;
  } else {
    startPos = 0;
    len += (offset) + 1;
  }
  if (offset + maxPadding >= src.length()) {
    len = src.length() - offset;
  }
  printf("scanner error:\n%d\t%s\n\t%s\n", pos.line, src.substr(startPos, len).c_str(), message.c_str());
  exit(1);
}

bool Scanner::Scanner::isLetter(char c) {
  char lowerCase = ('a' - 'A') | c;
  return 'a' <= lowerCase && lowerCase <= 'z' || c == '_';
}

bool Scanner::Scanner::isDigit(char c) {
  return '0' <= c && c <= '9';
}

bool Scanner::Scanner::isHex(char c) {
  char lowerCase = ('a' - 'A') | c;
  return '0' <= c && c <= '9' || 'a' <= lowerCase && lowerCase <= 'f';
}

void Scanner::Scanner::next() {
  offset++;
  if (offset == src.length()) {
    ch = -1;
  } else {
    ch = src[offset];
  }
}

void Scanner::Scanner::skipWhiteSpace() {
  while (ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r') {
    next();
  }
}

char Scanner::Scanner::peek() {
  if (offset + 1 < src.length()) {
    return src[offset + 1];
  }
  return 0;
}

std::string Scanner::Scanner::scanIdentifier() {
  int initialOffset = offset;
  next();
  while (isLetter(ch) || isDigit(ch)) {
    next();
  }
  return src.substr(initialOffset, (offset - initialOffset));
}

void Scanner::Scanner::scanDigits(int base, int& index) {
  if (base <= 10) {
    char max = '0' + base;  // maximum number allowed for this number system
    while (isDigit(ch) || ch == '_') {
      if (ch >= max && index < 0 && ch != '_') {
        index = offset;
      }

      next();
    }
  } else {
    while (isHex(ch) || ch == '_') {
      next();
    }
  }
}

std::pair<Token::TokenType, std::string> Scanner::Scanner::scanNumber() {
  int initialOffset = offset;
  Token::TokenType tokTy;
  std::string value = "";

  int base = 10;
  int invalidDigitIndex = -1;

  if (ch != '.') {
    tokTy = Token::INT;
    if (ch == '0') {
      next();
      switch (ch) {
        case 'x':
          next();
          base = 16;
          break;
        case 'b':
          next();
          base = 2;
        default:
          break;
      }
    }

    scanDigits(base, invalidDigitIndex);
  }

  if (ch == '.') {
    tokTy = Token::FLOAT;
    if (base != 10) {
      error(offset, "floating point numbers are only permitted in base 10");
    }
    next();
    scanDigits(base, invalidDigitIndex);
  }

  value = src.substr(initialOffset, (offset - initialOffset));
  if (invalidDigitIndex >= 0) {
    std::string litName = "";
    switch (base) {
      case 2:
        litName = "binary number";
        break;
      case 10:
        litName = "decimal number";
        break;
      case 16:
        litName = "hexidecimal number";
        break;
      default:
        litName = "decimal number";
        break;
    }
    std::string msg = fmt("invalid digit %s in %s", value[invalidDigitIndex - initialOffset], litName);
    error(invalidDigitIndex, msg);
  }

  return std::pair(tokTy, value);
}

void Scanner::Scanner::scanEscape(char quote) {
  int initialOffset = offset;

  switch (peek()) {
      // case quote:
    case 'r':
    case 't':
    case 'n':
      return next();
    default:
      return error(initialOffset, "unknown escape sequence");
  }
}

std::string Scanner::Scanner::scanString() {
  int initialOffset = offset;
  next();  // consume "

  while (true) {
    if (ch == '\n' || ch < 0) {
      error(initialOffset, "string literal not terminated (are you missing a \"?)");
      break;
    }
    if (ch == '"') {
      next();
      break;
    }
    if (ch == '\\') {
      scanEscape('"');
    }
    next();
  }

  return src.substr(initialOffset, (offset - initialOffset));
}

std::string Scanner::Scanner::scanChar() {
  int initialOffset = offset;
  next();  // consume '

  int n = 0;

  while (true) {
    if (ch == '\n' || ch < 0) {
      error(initialOffset, "char literal not terminated (are you missing a '?)");
      break;
    }
    if (ch == '\'') {
      next();
      break;
    }
    if (ch == '\\') {
      scanEscape('\'');
    }
    next();
    n++;
  }

  if (n != 1) {
    error(initialOffset, "invalid char literal");
  }

  return src.substr(initialOffset, (offset - initialOffset));
}

std::string Scanner::Scanner::scanComment(bool singleLine) {
  next();
  next();

  int initialOffset = offset;

  if (singleLine) {
    while (ch != '\n') {
      next();
    }
  } else {
    while (ch != '*' && peek() != '/') {
      next();
    }
  }

  std::string content = src.substr(initialOffset, (offset - initialOffset));
  if (!singleLine) {
    // Consume */
    next();
    next();
  }
  return content;
}

Token::Token Scanner::Scanner::scan() {
  skipWhiteSpace();

  Token::TokenType tokType = Token::TokenType::ILLEGAL;
  std::string value = "";

  if (isLetter(ch)) {
    value = scanIdentifier();
    if (value.length() > 1) {
      tokType = Token::lookup(value);
    } else {
      tokType = Token::IDENT;
    }
  } else if (isDigit(ch) || (ch == '.' && isDigit(peek()))) {
    auto result = scanNumber();
    tokType = result.first;
    value = result.second;
  } else {
    switch (ch) {
      case ' ':
        next();
        break;
      case '\n':
        pos.line++;
        break;
      case ';':
        tokType = Token::SEMICOLON;
        value = ";";
        break;
      case '"':
        tokType = Token::STRING_LIT;
        value = scanString();
        break;
      case '\'':
        tokType = Token::CHAR;
        value = scanChar();
        break;
      case '(':
        tokType = Token::LPAREN;
        value = "(";
        break;
      case ')':
        tokType = Token::RPAREN;
        value = ")";
        break;
      case '{':
        tokType = Token::LBRACE;
        value = "{";
        break;
      case '}':
        tokType = Token::RBRACE;
        value = "}";
        break;
      case ',':
        tokType = Token::COMMA;
        value = ",";
        break;
      case '&':
        tokType = Token::AMPERSAND;
        value = "&";
        break;
      case '*':
        tokType = Token::ASTERISK;
        value = "*";
        break;
      case '+':
        tokType = Token::PLUS;
        value = "+";
        break;
      case '=':
        tokType = Token::EQ;
        value = "=";
        break;
      case '-':
        tokType = Token::MINUS;
        value = "-";
        if (peek() == '>') {
          tokType = Token::ARROW;
          value = "->";
          next();
        }
        break;
      case '/':
        tokType = Token::COMMENT;
        if (peek() == '/') {
          value = scanComment(true);
        } else if (peek() == '*') {
          value = scanComment(false);
        }
      default:
        break;
    }
    next();
  }

  skipWhiteSpace();

  Token::Token tok;
  tok.pos = {"test.pi", 1, 1};
  tok.type = tokType;
  tok.value = value;
  return tok;
  // return {{"", -1, -1}, tokType, value};
}

void Scanner::Scanner::tokenize() {
  while (ch != -1 && offset < src.length()) {
    auto token = scan();
    tokens.push_back(token);
  }

  Token::Token tok;
  tok.pos = {"test.pi", 1, 1};
  tok.type = Token::_EOF;
  tok.value = "EOF";
  tokens.push_back(tok);
}