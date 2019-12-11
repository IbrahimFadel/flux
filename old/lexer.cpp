#include "lexer.h"
#include <iostream>
#include <fstream>
#include <vector>
#include <ctype.h>

using namespace Lexer;

char *get_file_input(const char *path)
{
  std::ifstream is(path, std::ifstream::binary);
  if (is)
  {
    is.seekg(0, is.end);
    int length = is.tellg();
    is.seekg(0, is.beg);
    char *buffer = new char[length];
    is.read(buffer, length);
    is.close();

    return buffer;
  }
  return {};
};

Token create_token(int type, std::string value)
{
  Token tok;
  tok.type = type;
  tok.value = value;
  return tok;
}

std::vector<Token> generate_tokens(std::string input)
{
  int chars_skipped = 0;
  int len = input.length();

  std::vector<int> numbers = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9};

  std::vector<Token> tokens;
  std::string token;

  int chars_skipped_iterator = 0;

  for (int i = 0; i < len; i++)
  {
    std::string number;
    char c = input[i];

    for (int j = chars_skipped_iterator; j < chars_skipped; j++)
    {
      if (j + 1 == chars_skipped)
      {
        chars_skipped = 0;
        chars_skipped_iterator = 0;
      }
      token.clear();
      chars_skipped_iterator++;
      goto end;
    }

    token += c;

    if (c == ' ')
    {
      if (token != " ")
      {
        tokens.push_back(create_token(Types::id, token));
      }
      token.clear();
      continue;
    }
    else if (c == '\n')
    {
      if (token.size() > 1)
      {
        tokens.push_back(create_token(Types::id, token));
      }
      token.clear();
      continue;
    }
    else if (c == '\t')
    {
      if (token.size() > 1)
      {
        tokens.push_back(create_token(Types::id, token));
      }
      token.clear();
      continue;
    }
    else if (c == '(')
    {
      tokens.push_back(create_token(Types::opr, token.substr(token.length() - 1)));
      token.clear();
      continue;
    }
    else if (c == ')')
    {
      if (token.size() > 1)
      {
        tokens.push_back(create_token(Types::id, token.substr(0, token.length() - 1)));
        tokens.push_back(create_token(Types::cpr, token.substr(token.length() - 1)));
      }
      else
      {
        tokens.push_back(create_token(Types::cpr, token.substr(token.length() - 1)));
      }
      token.clear();
      continue;
    }
    else if (c == '"')
    {
      if (token.length() > 1)
      {
        tokens.push_back(create_token(Types::id, token.substr(0, token.length() - 1)));
        tokens.push_back(create_token(Types::odq, "\""));
      }
      else
      {
        tokens.push_back(create_token(Types::cdq, "\""));
      }
      token.clear();
      continue;
    }

    if (isdigit(c))
    {
      number.push_back(c);
      for (int j = 1; j < len - i; j++)
      {
        if (!isdigit(input[i + 1]))
        {
          break;
        }
        else if (!isdigit(input[i + j - 1]))
        {
          break;
        }
        else if (isdigit(input[i + j]))
        {
          chars_skipped++;
          number.push_back(input[i + j]);
        }
      }
      tokens.push_back(create_token(Types::num, number));
      token.clear();
      continue;
    }

    if (token == "fn")
    {
      tokens.push_back(create_token(Types::kw, token));
      token.clear();
    }
    else if (token == "main")
    {
      tokens.push_back(create_token(Types::id, token));
      token.clear();
    }
    else if (token == "{")
    {
      tokens.push_back(create_token(Types::id, token));
      token.clear();
    }
    else if (token == "}")
    {
      tokens.push_back(create_token(Types::id, token));
      token.clear();
    }
    else if (token == ";")
    {
      tokens.push_back(create_token(Types::eol, token));
      token.clear();
    }
    else if (token == "let")
    {
      tokens.push_back(create_token(Types::kw, token));
      token.clear();
    }
    else if (token == "=")
    {
      tokens.push_back(create_token(Types::op, token));
      token.clear();
    }
    else if (token == "print")
    {
      tokens.push_back(create_token(Types::kw, token));
      token.clear();
    }
    else if (token == "+")
    {
      tokens.push_back(create_token(Types::op, token));
      token.clear();
    }
    else if (token == "*")
    {
      tokens.push_back(create_token(Types::op, token));
      token.clear();
    }

  end:;
  }

  return tokens;
};