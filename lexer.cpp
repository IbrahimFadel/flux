#include <iostream>
#include <vector>
#include <string>
#include <ctype.h>
#include "lexer.h"

using namespace Lexer;

using std::cout;
using std::endl;
using std::string;
using std::vector;

Token create_token(int type, string value)
{
  Token token;
  token.type = type;
  token.value = value;
  return token;
}

vector<Token> generate_tokens(string input)
{
  vector<Token> tokens;
  string keywords[10] = {"fn", "main", "let", "print", "int", "float", "string", "object", "class", "while"};
  string token;
  bool is_string = false;
  string number;
  int number_chars_skipped = 0;
  int chars_skipped = 0;

  for (int i = 0; i < input.length(); i++)
  {
    char c = input[i];

    for (int j = chars_skipped; j < number_chars_skipped; j++)
    {
      if (j + 1 == number_chars_skipped)
      {
        number_chars_skipped = 0;
        chars_skipped = 0;
        token.clear();
        continue;
      }
      chars_skipped++;
      goto end;
    }

    if (c == ' ' || c == '\n' || c == '\t')
    {
      if (is_string)
      {
        token += c;
        continue;
      }
      if (token.length() > 1)
      {
        tokens.push_back(create_token(Types::id, token));
      }
      token.clear();
      continue;
    }
    else if (c == '\"')
    {
      if (is_string)
      {
        token += "\"";
        tokens.push_back(create_token(Types::lit, token));
        token.clear();
        is_string = !is_string;
        continue;
      }
      is_string = !is_string;
    }
    else if (c == ',')
    {
      if (is_string)
      {
        token += c;
        continue;
      }
      if (token.length() > 1)
      {
        tokens.push_back(create_token(Types::id, token));
        tokens.push_back(create_token(Types::sep, ","));
        token.clear();
        continue;
      }
    }
    else if (c == ')')
    {
      if (is_string)
      {
        token += c;
        continue;
      }
      if (token.length() > 1)
      {
        tokens.push_back(create_token(Types::id, token.substr(0, token.length())));
        tokens.push_back(create_token(Types::sep, ")"));
        token.clear();
        continue;
      }
    }

    token += c;

    if (is_string)
    {
      continue;
    }

    if (isdigit(c))
    {
      for (int j = 0; j < input.length() - i; j++)
      {
        if (isdigit(input[i + j]))
        {
          number += input[i + j];
          number_chars_skipped++;
        }
        else
        {
          tokens.push_back(create_token(Types::lit, number));
          number.clear();
          token.clear();
          goto end;
        }
      }
    }

    for (int j = 0; j < sizeof(keywords) / sizeof(keywords[0]); j++)
    {
      if (keywords[j] == token)
      {
        tokens.push_back(create_token(Types::kw, token));
        token.clear();
        continue;
      }
    }
    if (token == "(" || token == ")" || token == "{" || token == "}")
    {
      tokens.push_back(create_token(Types::sep, token));
      token.clear();
      continue;
    }
    else if (token == ";")
    {
      tokens.push_back(create_token(Types::eol, token));
      token.clear();
      continue;
    }
    else if (token == "*" || token == "/" || token == "+" || token == "-")
    {
      tokens.push_back(create_token(Types::op, token));
      token.clear();
      continue;
    }
    else if (token == "true" || token == "false")
    {
      tokens.push_back(create_token(Types::lit, token));
      token.clear();
      continue;
    }

  end:;
  }

  // for (int i = 0; i < tokens.size(); i++)
  // {
  //   // cout << tokens[i].type << " => " << tokens[i].value << endl;
  //   cout << tokens[i].value << ' ';
  // }
  // cout << endl;

  return tokens;
}