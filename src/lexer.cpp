#include "lexer.h"

vector<unique_ptr<Token>> get_tokens(const std::string content)
{
  vector<unique_ptr<Token>> tokens;

  std::string token = "";

  unsigned int row = 0;
  unsigned int col = 0;

  int i = 0;
  for (char const &c : content)
  {
    cout << token << endl;

    switch (c)
    {
    case ' ':
      add_token(token, tokens, row, col);
      break;
    case ':':
      add_token(token, tokens, row, col);

    default:
      break;
    }

    token += c;
  }

  return tokens;
}

void add_token(std::string &token, vector<unique_ptr<Token>> &tokens, unsigned int row, unsigned int col)
{
  unique_ptr<Token> tok = std::make_unique<Token>();

  trim(token);

  if (token == "let")
  {
    tok->type = Token_Types::tok_let;
  }
  else if (token == "int")
  {
    tok->type = Token_Types::tok_int;
  }
  else if (token == ":")
  {
    tok->type = Token_Types::tok_colon;
  }
  else if (token == "=")
  {
    tok->type = Token_Types::tok_eq;
  }
  else
  {
    tok->type = Token_Types::tok_identifier;
    tok->value = token;
  }

  token.clear();
  tokens.push_back(std::move(tok));
}

void print_tokens(vector<unique_ptr<Token>> tokens)
{
  cout << "------------ Tokens ------------" << endl;
  for (auto &token : tokens)
  {
    cout << token->type << endl;
  }
  cout << "--------------------------------" << endl;
}