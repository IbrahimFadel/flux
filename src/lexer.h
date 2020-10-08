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

enum Token_Types
{
  tok_let,
  tok_fn,

  tok_int,

  tok_colon,
  tok_semicolon,
  tok_comma,
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
  tok_identifier
};

struct Token
{
  Token_Types type;
  std::string value;
  unsigned int row;
  unsigned int col;
};

static vector<std::string> keywords = {
    "let"};

static std::string file_content;
static unsigned int file_content_pos;

vector<shared_ptr<Token>> get_tokens(const std::string content);
void print_tokens(vector<shared_ptr<Token>> tokens);

void add_token(std::string &token, vector<shared_ptr<Token>> &tokens, unsigned int row, unsigned int col, bool single_char_tok = false);

static inline bool is_number(const std::string &s)
{
  return !s.empty() && std::find_if(s.begin(),
                                    s.end(), [](char c) { return !std::isdigit(c); }) == s.end();
}

static inline void ltrim(std::string &s)
{
  s.erase(s.begin(), std::find_if(s.begin(), s.end(), [](int ch) {
            return !std::isspace(ch);
          }));
}

// trim from end (in place)
static inline void rtrim(std::string &s)
{
  s.erase(std::find_if(s.rbegin(), s.rend(), [](int ch) {
            return !std::isspace(ch);
          }).base(),
          s.end());
}

// trim from both ends (in place)
static inline void trim(std::string &s)
{
  ltrim(s);
  rtrim(s);
}

#endif