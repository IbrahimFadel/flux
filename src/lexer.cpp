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

int line_number = 0;
int line_position = -1;

Token create_token(int type, string value, int line_pos)
{
  Token tok;
  tok.type = type;
  tok.value = value;
  tok.line_number = line_number;
  tok.line_position = line_pos;
  return tok;
}

vector<Token> generate_tokens(vector<string> input)
{
  vector<Token> tokens;
  string token;
  string number;

  int chars_skipped = 0;
  int number_chars_skipped = 0;
  bool is_string = false;
  string keywords[13] = {"fn", "main", "let", "print", "int", "float", "string", "object", "class", "while", "if", "continue", "break"};

  for (int i = 0; i < input.size(); i++)
  {
    string line = input[i];
    for (int j = 0; j < line.length(); j++)
    {
      line_position++;
      char c = line[j];

      for (int x = chars_skipped; x < number_chars_skipped; x++)
      {
        if (x + 1 == number_chars_skipped)
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
        if (token.length() > 0)
        {
          tokens.push_back(create_token(Types::id, token, line_position - token.length()));
        }
        token.clear();
        continue;
      }
      else if (c == '\"')
      {
        if (is_string)
        {
          token += "\"";
          tokens.push_back(create_token(Types::lit, token, line_position - (token.length() - 1)));
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
        if (token.length() > 0)
        {
          tokens.push_back(create_token(Types::id, token, line_position - token.length()));
          tokens.push_back(create_token(Types::sep, ",", line_position - token.length()));
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
        if (token.length() > 0)
        {
          tokens.push_back(create_token(Types::id, token.substr(0, token.length()), line_position - token.length()));
          tokens.push_back(create_token(Types::sep, ")", line_position - token.length()));
          token.clear();
          continue;
        }
        else
        {
          tokens.push_back(create_token(Types::sep, ")", line_position - token.length()));
          token.clear();
          continue;
        }
      }
      else if (c == '(')
      {
        if (is_string)
        {
          token += c;
          continue;
        }
        if (token.length() > 1)
        {
          tokens.push_back(create_token(Types::id, token.substr(0, token.length()), line_position - token.length() + 1));
          tokens.push_back(create_token(Types::sep, "(", line_position));
          token.clear();
          continue;
        }
        else
        {
          tokens.push_back(create_token(Types::sep, "(", line_position));
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
        if (line[j - 1] == '-')
        {
          number += "-";
        }
        for (int x = 0; x < line.length() - j; x++)
        {
          if (isdigit(line[j + x]))
          {
            number += line[j + x];
            number_chars_skipped++;
            if (j + x + 1 == line.length())
            {
              tokens.push_back(create_token(Types::lit, number, line_position - token.length() + 1));
              number.clear();
              token.clear();
              goto end;
            }
          }
          else
          {
            tokens.push_back(create_token(Types::lit, number, line_position - token.length() + 1));
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
          tokens.push_back(create_token(Types::kw, token, line_position - (token.length() - 1)));
          token.clear();
          continue;
        }
      }
      if (token == "{" || token == "}")
      {
        tokens.push_back(create_token(Types::sep, token, line_position));
        token.clear();
        continue;
      }
      else if (token == ";")
      {
        tokens.push_back(create_token(Types::eol, token, line_position));
        token.clear();
        continue;
      }
      else if (token == "*" || token == "/" || token == "+" || token == "-" || token == "<" || token == ">" || token == "==" || token == "%" || token == "&&" || token == "||" || token == "%" || token == "!=")
      {
        if (token == "<" || token == ">")
        {
          if (line[j + 1] == '=')
          {
            continue;
          }
        }
        else if (token == "-")
        {
          if (isdigit(line[j + 1]))
          {
            continue;
          }
        }
        tokens.push_back(create_token(Types::op, token, line_position));
        token.clear();
        continue;
      }
      else if (token == "true" || token == "false")
      {
        tokens.push_back(create_token(Types::lit, token, line_position - token.length()));
        token.clear();
        continue;
      }

    end:;
    }

    line_position = -1;
    line_number++;
  }

  return tokens;
}