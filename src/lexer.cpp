#include "lexer.h"

vector<shared_ptr<Token>> get_tokens(const std::string content)
{
  file_content_pos = 0;
  file_content = content;

  vector<shared_ptr<Token>> tokens;

  std::string token = "";

  unsigned int row = 1;
  unsigned int col = 1;

  for (auto &c : content)
  {
    if (c != ' ')
      token += c;
    switch (c)
    {
    case ' ':
      add_token(token, tokens, row, col);
      break;
    case '\n':
      row++;
      col = 1;
      break;
    case '\t':
      break;
    case ':':
      add_token(token, tokens, row, col, true);
      break;
    case ';':
      add_token(token, tokens, row, col, true);
      break;
    case ',':
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
    col++;
  }

  shared_ptr<Token> eof_tok = std::make_shared<Token>();
  eof_tok->type = Token_Types::tok_eof;
  tokens.push_back(std::move(eof_tok));

  return tokens;
}

void add_token(std::string &token, vector<shared_ptr<Token>> &tokens, unsigned int row, unsigned int col, bool single_char_tok)
{
  trim(token);

  if (token.size() == 0)
    return;
  shared_ptr<Token> tok = std::make_shared<Token>();

  if (single_char_tok)
  {
    std::string spliced = token.substr(0, token.size() - 1);
    add_token(spliced, tokens, row, col);
    token = token.back();
    col++;
  }

  if (token == "let")
  {
    tok->type = Token_Types::tok_let;
  }
  else if (token == "fn")
  {
    tok->type = Token_Types::tok_fn;
  }
  else if (token == "return")
  {
    tok->type = Token_Types::tok_return;
  }
  else if (token == "i64")
  {
    tok->type = Token_Types::tok_i64;
  }
  else if (token == "i32")
  {
    tok->type = Token_Types::tok_i32;
  }
  else if (token == "i16")
  {
    tok->type = Token_Types::tok_i16;
  }
  else if (token == "i8")
  {
    tok->type = Token_Types::tok_i8;
  }
  else if (token == "toi64")
  {
    tok->type = Token_Types::tok_toi64;
  }
  else if (token == "toi32")
  {
    tok->type = Token_Types::tok_toi32;
  }
  else if (token == "toi16")
  {
    tok->type = Token_Types::tok_toi16;
  }
  else if (token == "toi8")
  {
    tok->type = Token_Types::tok_toi8;
  }
  else if (token == "float")
  {
    tok->type = Token_Types::tok_float;
  }
  else if (token == "double")
  {
    tok->type = Token_Types::tok_double;
  }
  else if (token == "bool")
  {
    tok->type = Token_Types::tok_bool;
  }
  else if (token == ":")
  {
    tok->type = Token_Types::tok_colon;
  }
  else if (token == ";")
  {
    tok->type = Token_Types::tok_semicolon;
  }
  else if (token == ",")
  {
    tok->type = Token_Types::tok_comma;
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

  tok->value = token;

  tok->row = row;
  tok->col = col - tok->value.size();

  // cout << tok->row << ' ' << tok->col << endl;

  token.clear();
  tokens.push_back(std::move(tok));
}

void print_tokens(vector<shared_ptr<Token>> tokens)
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
    case Token_Types::tok_return:
      cout << "return" << endl;
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
    case Token_Types::tok_comma:
      cout << "," << endl;
      break;
    case Token_Types::tok_i64:
      cout << "i64" << endl;
      break;
    case Token_Types::tok_i32:
      cout << "i32" << endl;
      break;
    case Token_Types::tok_i16:
      cout << "i16" << endl;
      break;
    case Token_Types::tok_i8:
      cout << "i8" << endl;
      break;
    case Token_Types::tok_toi64:
      cout << "toi64" << endl;
      break;
    case Token_Types::tok_toi32:
      cout << "toi32" << endl;
      break;
    case Token_Types::tok_toi16:
      cout << "toi16" << endl;
      break;
    case Token_Types::tok_toi8:
      cout << "toi8" << endl;
      break;
    case Token_Types::tok_float:
      cout << "float" << endl;
      break;
    case Token_Types::tok_double:
      cout << "double" << endl;
      break;
    case Token_Types::tok_bool:
      cout << "bool" << endl;
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
    case Token_Types::tok_plus:
      cout << "+" << endl;
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
    case Token_Types::tok_eof:
      cout << "EOF" << endl;
      break;

    default:
      break;
    }
  }
  cout << "--------------------------------" << endl;
}