#ifndef LEXER_H
#define LEXER_H

#include <iostream>
#include <vector>
#include <memory>
#include <algorithm>

using std::cout;
using std::endl;
using std::shared_ptr;
using std::vector;

enum Token_Type
{
  tok_let,
  tok_fn,
  tok_if,
  tok_for,
  tok_return,
  tok_import,
  tok_toi64,
  tok_toi32,
  tok_toi16,
  tok_toi8,

  tok_i64,
  tok_i32,
  tok_i16,
  tok_i8,
  tok_bool,
  tok_float,
  tok_double,
  tok_string,
  tok_object,

  tok_compare_eq,
  tok_compare_lt,
  tok_compare_gt,
  tok_and,
  tok_or,

  tok_colon,
  tok_semicolon,
  tok_comma,
  tok_period,
  tok_open_paren,
  tok_close_paren,
  tok_open_curly_bracket,
  tok_close_curly_bracket,

  tok_eq,
  tok_asterisk,
  tok_slash,
  tok_plus,
  tok_minus,
  tok_arrow,

  tok_number,
  tok_string_lit,
  tok_identifier,

  tok_eof
};

struct Token
{
  Token_Type type;
  std::string value;
  unsigned int row;
  unsigned int col;
};

void print_tokens(vector<shared_ptr<Token>> tokens);
vector<shared_ptr<Token>> get_tokens(vector<std::string> content);
static void handle_token(std::string &token, vector<shared_ptr<Token>> &tokens, int &row, int &col);
static void handle_single_char_token(char c, vector<shared_ptr<Token>> &tokens, int &row, int &col);
static void create_token(Token_Type type, std::string &token, std::vector<shared_ptr<Token>> &tokens, int &row, int &col);

static bool add_next_char = true;
static bool is_string = false;

static inline bool is_floating_point(const char *str)
{
  char *endptr = 0;
  strtod(str, &endptr);

  if (*endptr != '\0' || endptr == str)
    return false;
  return true;
}

static inline bool is_number(const std::string &s)
{
  if (is_floating_point(s.c_str()))
    return true;
  return !s.empty() && std::find_if(s.begin(),
                                    s.end(), [](char c) { return !std::isdigit(c); }) == s.end();
}

#endif