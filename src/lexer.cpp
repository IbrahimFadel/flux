#include "lexer.h"

vector<unique_ptr<Token>> get_tokens(const std::string content)
{
  file_content_pos = 0;
  file_content = content;

  vector<unique_ptr<Token>> tokens;

  std::string token = "";

  unsigned int row = 0;
  unsigned int col = 0;

  for (auto &c : content)
  {
    if (c != ' ')
      token += c;
    switch (c)
    {
    case ' ':
      add_token(token, tokens, row, col);
      break;
    case ':':
      add_token(token, tokens, row, col, true);
      break;
    case ';':
      add_token(token, tokens, row, col, true);
      break;
    case '(':
      add_token(token, tokens, row, col, true);
      break;
    case ')':
      add_token(token, tokens, row, col, true);
      break;
    case '{':
      add_token(token, tokens, row, col, true);
      break;
    case '}':
      add_token(token, tokens, row, col, true);
      break;
    default:
      break;
    }

    file_content_pos++;
  }

  return tokens;
}

void add_token(std::string &token, vector<unique_ptr<Token>> &tokens, unsigned int row, unsigned int col, bool single_char_tok)
{
  trim(token);

  if (token.size() == 0)
    return;
  unique_ptr<Token> tok = std::make_unique<Token>();

  if (single_char_tok)
  {
    std::string spliced = token.substr(0, token.size() - 1);
    add_token(spliced, tokens, row, col);
    token = token.back();
  }

  if (token == "let")
  {
    tok->type = Token_Types::tok_let;
  }
  else if (token == "fn")
  {
    tok->type = Token_Types::tok_fn;
  }
  else if (token == "int")
  {
    tok->type = Token_Types::tok_int;
  }
  else if (token == ":")
  {
    tok->type = Token_Types::tok_colon;
  }
  else if (token == ";")
  {
    tok->type = Token_Types::tok_semicolon;
  }
  else if (token == "=")
  {
    tok->type = Token_Types::tok_eq;
  }
  else if (is_number(token))
  {
    tok->type = Token_Types::tok_number;
    tok->value = token;
  }
  else if (token == "*")
  {
    tok->type = Token_Types::tok_asterisk;
  }
  else if (token == "/")
  {
    tok->type = Token_Types::tok_slash;
  }
  else if (token == "+")
  {
    tok->type = Token_Types::tok_plus;
  }
  else if (token == "->")
  {
    tok->type = Token_Types::tok_arrow;
  }
  else if (token == "-")
  {
    tok->type = Token_Types::tok_minus;
  }
  else if (token == "(")
  {
    tok->type = Token_Types::tok_open_paren;
  }
  else if (token == ")")
  {
    tok->type = Token_Types::tok_close_paren;
  }
  else if (token == "{")
  {
    tok->type = Token_Types::tok_open_curly_bracket;
  }
  else if (token == "}")
  {
    tok->type = Token_Types::tok_close_curly_bracket;
  }
  else
  {
    tok->type = Token_Types::tok_identifier;
    tok->value = token;
  }

  tok->row = row;
  tok->col = col;

  token.clear();
  tokens.push_back(std::move(tok));
}

void print_tokens(vector<unique_ptr<Token>> tokens)
{
  cout << "------------ Tokens ------------" << endl;
  for (auto &token : tokens)
  {
    cout << token->type << ' ';

    switch (token->type)
    {
    case Token_Types::tok_let:
      cout << "let" << endl;
      break;
    case Token_Types::tok_fn:
      cout << "fn" << endl;
      break;
    case Token_Types::tok_identifier:
      cout << token->value << endl;
      break;
    case Token_Types::tok_colon:
      cout << ":" << endl;
      break;
    case Token_Types::tok_semicolon:
      cout << ";" << endl;
      break;
    case Token_Types::tok_int:
      cout << "int" << endl;
      break;
    case Token_Types::tok_eq:
      cout << "=" << endl;
      break;
    case Token_Types::tok_number:
      cout << token->value << endl;
      break;
    case Token_Types::tok_asterisk:
      cout << "*" << endl;
      break;
    case Token_Types::tok_open_paren:
      cout << "(" << endl;
      break;
    case Token_Types::tok_close_paren:
      cout << ")" << endl;
      break;
    case Token_Types::tok_open_curly_bracket:
      cout << "{" << endl;
      break;
    case Token_Types::tok_close_curly_bracket:
      cout << "}" << endl;
      break;
    case Token_Types::tok_arrow:
      cout << "->" << endl;
      break;

    default:
      break;
    }
  }
  cout << "--------------------------------" << endl;
}