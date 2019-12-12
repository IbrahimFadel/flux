#include <iostream>
#include <vector>
#include <string>
// #include <cstring>
#include <ctype.h>
#include "lexer.h"

using namespace Lexer;

using std::cout;
using std::endl;
using std::string;
using std::vector;

int line_number = 0;
int line_position = 0;

Token create_token(int type, string value)
{
  Token token;
  token.type = type;
  token.value = value;
  token.line_number = line_number;
  token.line_position = line_position;
  return token;
}

vector<Token> generate_tokens(vector<string> input)
{
  vector<Token> tokens;
  string token;
  string number;

  int chars_skipped = 0;
  int number_chars_skipped = 0;
  bool is_string = false;
  string keywords[10] = {"fn", "main", "let", "print", "int", "float", "string", "object", "class", "while"};

  for (int i = 0; i < input.size(); i++)
  {
    string line = input[i];
    for (int j = 0; j < line.length(); j++)
    {
      char c = line[j];

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
        else
        {
          tokens.push_back(create_token(Types::sep, ")"));
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
          tokens.push_back(create_token(Types::id, token.substr(0, token.length())));
          tokens.push_back(create_token(Types::sep, "("));
          token.clear();
          continue;
        }
        else
        {
          tokens.push_back(create_token(Types::sep, "("));
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
        for (int x = 0; x < line.length() - j; x++)
        {
          if (isdigit(line[j + x]))
          {
            number += line[j + x];
            number_chars_skipped++;
            if (j + x + 1 == line.length())
            {
              tokens.push_back(create_token(Types::lit, number));
              number.clear();
              token.clear();
              goto end;
            }
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
      if (token == "{" || token == "}")
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
      else if (token == "*" || token == "/" || token == "+" || token == "-" || token == "<" || token == ">")
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

    end:
      line_position++;
    }

    line_position = 0;
    line_number++;
  }

  return tokens;
}

// vector<Token> generate_tokens(vector<string> input)
// {
//   vector<Token> tokens;
//   string keywords[10] = {"fn", "main", "let", "print", "int", "float", "string", "object", "class", "while"};
//   string token;
//   bool is_string = false;
//   string number;
//   int number_chars_skipped = 0;
//   int chars_skipped = 0;

//   for (int i = 0; i < input.size(); i++)
//   {
//     string line = input[i];
//     for (int j = 0; i < line.length(); j++)
//     {
//       char c = line[j];

//       for (int j = chars_skipped; j < number_chars_skipped; j++)
//       {
//         if (j + 1 == number_chars_skipped)
//         {
//           number_chars_skipped = 0;
//           chars_skipped = 0;
//           token.clear();
//           continue;
//         }
//         chars_skipped++;
//         goto end;
//       }

//       if (c == ' ' || c == '\n' || c == '\t')
//       {
//         if (is_string)
//         {
//           token += c;
//           continue;
//         }
//         if (token.length() > 1)
//         {
//           tokens.push_back(create_token(Types::id, token));
//         }
//         token.clear();
//         continue;
//       }
//       else if (c == '\"')
//       {
//         if (is_string)
//         {
//           token += "\"";
//           tokens.push_back(create_token(Types::lit, token));
//           token.clear();
//           is_string = !is_string;
//           continue;
//         }
//         is_string = !is_string;
//       }
//       else if (c == ',')
//       {
//         if (is_string)
//         {
//           token += c;
//           continue;
//         }
//         if (token.length() > 1)
//         {
//           tokens.push_back(create_token(Types::id, token));
//           tokens.push_back(create_token(Types::sep, ","));
//           token.clear();
//           continue;
//         }
//       }
//       else if (c == ')')
//       {
//         if (is_string)
//         {
//           token += c;
//           continue;
//         }
//         if (token.length() > 1)
//         {
//           tokens.push_back(create_token(Types::id, token.substr(0, token.length())));
//           tokens.push_back(create_token(Types::sep, ")"));
//           token.clear();
//           continue;
//         }
//         else
//         {
//           tokens.push_back(create_token(Types::sep, ")"));
//           token.clear();
//           continue;
//         }
//       }
//       else if (c == '(')
//       {
//         if (is_string)
//         {
//           token += c;
//           continue;
//         }
//         if (token.length() > 1)
//         {
//           tokens.push_back(create_token(Types::id, token.substr(0, token.length())));
//           tokens.push_back(create_token(Types::sep, "("));
//           token.clear();
//           continue;
//         }
//         else
//         {
//           tokens.push_back(create_token(Types::sep, "("));
//           token.clear();
//           continue;
//         }
//       }

//       token += c;

//       if (is_string)
//       {
//         continue;
//       }

//       if (isdigit(c))
//       {
//         for (int j = 0; j < line.length() - i; j++)
//         {
//           if (isdigit(line[i + j]))
//           {
//             number += line[i + j];
//             number_chars_skipped++;
//             if (i + j + 1 == line.length())
//             {
//               tokens.push_back(create_token(Types::lit, number));
//               number.clear();
//               token.clear();
//               goto end;
//             }
//           }
//           else
//           {
//             tokens.push_back(create_token(Types::lit, number));
//             number.clear();
//             token.clear();
//             goto end;
//           }
//         }
//       }

//       for (int j = 0; j < sizeof(keywords) / sizeof(keywords[0]); j++)
//       {
//         if (keywords[j] == token)
//         {
//           tokens.push_back(create_token(Types::kw, token));
//           token.clear();
//           continue;
//         }
//       }
//       if (token == "{" || token == "}")
//       {
//         tokens.push_back(create_token(Types::sep, token));
//         token.clear();
//         continue;
//       }
//       else if (token == ";")
//       {
//         tokens.push_back(create_token(Types::eol, token));
//         token.clear();
//         continue;
//       }
//       else if (token == "*" || token == "/" || token == "+" || token == "-" || token == "<" || token == ">")
//       {
//         tokens.push_back(create_token(Types::op, token));
//         token.clear();
//         continue;
//       }
//       else if (token == "true" || token == "false")
//       {
//         tokens.push_back(create_token(Types::lit, token));
//         token.clear();
//         continue;
//       }

//     end:
//       line_position++;
//     }
//     line_number++;
//   }
//   // for (int i = 0; i < line.length(); i++)
//   // {
//   // }

//   return tokens;
// }