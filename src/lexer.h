#ifndef LEXER_H
#define LEXER_H

#include <iostream>
#include <vector>
#include <memory>

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
  "u8"
};

static const std::vector<std::string> operators = {
  "+",
  "-",
  "*",
  "/",
  ":",
  "&&",
  "||",
  ">",
  "<",
  "=",
  "==",
  "<=",
  ">="
};

void print_tokens(std::vector<std::shared_ptr<Token>> &);

std::vector<std::shared_ptr<Token>> get_tokens(std::string);
void add_token(std::string &, std::vector<std::shared_ptr<Token>> &, int, int, bool);
std::shared_ptr<Token> create_token(std::string, int, int, int, int);
bool is_keyword(std::string);
bool is_operator(std::string);
bool is_literal(std::string, bool);

#endif