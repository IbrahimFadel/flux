#ifndef LEXER_H
#define LEXER_H

#include <iostream>
#include <vector>
#include <memory>
#include <algorithm>

using std::cout;
using std::endl;

enum Lexer_State
{
  normal,
  string,
  comment
};

enum String_State
{
  double_quotes,
  single_quotes,
  none
};

enum Token_Types
{
  kw,
  id,
  op,
  lit,
  sep,
  eol
};

struct Token
{
  std::string value;
  int type;
  int start_row;
  int start_col;
  int end_row;
  int end_col;
};

static const std::vector<std::string> keywords = {
    "const",
    "let",
    "fn",
    "int",
    "string",
    "void",
    "array",
    "return"};

static const std::vector<std::string> types = {
    "int",
    "string",
    "void",
    "array"};

static const std::vector<std::string>
    operators = {"+", "-", "*", "/", ":", "&&", "||", ">", "<", "=", "==", "<=", ">="};

static const std::vector<std::string> seperators = {
    "(",
    ")",
    "{",
    "}",
    "[",
    "]",
    "<",
    ">"};

void print_tokens(std::vector<std::shared_ptr<Token>> &);

std::vector<std::shared_ptr<Token>> get_tokens(std::string);
void add_token(std::string &, std::vector<std::shared_ptr<Token>> &, int, int, int, int);
std::shared_ptr<Token> create_token(std::string, int, int, int, int);
bool is_keyword(std::string);
bool is_operator(std::string);
bool is_literal(std::string, int);
bool is_number(std::string);
bool is_type(std::string);
bool is_seperator(std::string);

#endif