#ifndef LEXER_H
#define LEXER_H

#include <iostream>
#include <vector>
#include <memory>
#include <algorithm>

using std::cout;
using std::endl;
using std::shared_ptr;
using std::string;
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
  tok_ampersand,

  tok_number,
  tok_string_lit,
  tok_identifier,

  tok_eof
};

struct Token
{
  int row, col;
  string value;
  Token_Type type;
};

typedef vector<shared_ptr<Token>> Tokens;

static int row = 0, col = 0;

vector<shared_ptr<Token>> tokenize(vector<string> content);
static void add_token(string &token, vector<shared_ptr<Token>> &tokens, bool is_single_char_token = false, char single_char_token = '\0');
static shared_ptr<Token> create_token(string token);
static Token_Type get_token_type(string token);

static bool is_number(const string &token);
static bool is_floating_point(const char *str);

void print_tokens(const Tokens &tokens);

#endif