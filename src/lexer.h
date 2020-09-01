#ifndef LEXER_H
#define LEXER_H

#include <iostream>
#include <vector>
#include <memory>
#include <algorithm>

using std::cout;
using std::endl;
using std::unique_ptr;
using std::vector;

enum Token_Types
{
  tok_let,

  tok_int,

  tok_colon,
  tok_eq,

  tok_number,
  tok_identifier
};

struct Token
{
  Token_Types type;
  std::string value;
};

static vector<std::string> keywords = {
    "let"};

vector<unique_ptr<Token>> get_tokens(const std::string content);
void print_tokens(vector<unique_ptr<Token>> tokens);

void add_token(std::string &token, vector<unique_ptr<Token>> &tokens, unsigned int row, unsigned int col);

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