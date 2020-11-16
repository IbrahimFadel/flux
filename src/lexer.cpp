#include "lexer.h"

vector<shared_ptr<Token>> get_tokens(std::vector<std::string> content)
{
  vector<shared_ptr<Token>> tokens;
  std::string token;
  int row = 1, col = 1;
  is_string = false;
  for (auto &line : content)
  {
    for (auto &c : line)
    {
      add_next_char = true;

      if (is_string && c != '"')
      {
        token += c;
        continue;
      }

      switch (c)
      {
      case ' ':
        handle_token(token, tokens, row, col);
        break;
      case '\n':
        handle_token(token, tokens, row, col);
        break;
      case '(':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case ')':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '{':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '}':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case ';':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case ':':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case ',':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '.':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '=':
      {
        // cout << line[col] << endl;
        // if (line[col] != '=')
        // {
        //   cout << "doing" << endl;
        //   handle_token(token, tokens, row, col);
        //   handle_single_char_token(c, tokens, row, col);
        // }
        break;
      }
      break;
      case '*':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '/':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '+':
        handle_token(token, tokens, row, col);
        handle_single_char_token(c, tokens, row, col);
        break;
      case '-':
        if (line[col] != '>')
        {
          handle_token(token, tokens, row, col);
          handle_single_char_token(c, tokens, row, col);
        }
        break;
      case '"':
        if (is_string)
        {
          handle_token(token, tokens, row, col);
          is_string = false;
          add_next_char = false;
        }
        else
        {
          is_string = true;
          add_next_char = false;
        }
        break;
      default:
        break;
      }

      if (add_next_char)
      {
        if (c != ' ')
        {
          token += c;
        }
      }

      col++;
    }
    row++;
    col = 1;
  }

  token = "EOF";
  col = 4;
  create_token(Token_Type::tok_eof, token, tokens, row, col);

  return tokens;
}

void handle_token(std::string &token, vector<shared_ptr<Token>> &tokens, int &row, int &col)
{
  if (token.size() < 1)
    return;
  if (token == "if")
  {
    create_token(Token_Type::tok_if, token, tokens, row, col);
  }
  else if (token == "for")
  {
    create_token(Token_Type::tok_for, token, tokens, row, col);
  }
  else if (token == "fn")
  {
    create_token(Token_Type::tok_fn, token, tokens, row, col);
  }
  else if (token == "return")
  {
    create_token(Token_Type::tok_return, token, tokens, row, col);
  }
  else if (token == "import")
  {
    create_token(Token_Type::tok_import, token, tokens, row, col);
  }
  else if (token == "==")
  {
    create_token(Token_Type::tok_compare_eq, token, tokens, row, col);
  }
  else if (token == "<")
  {
    create_token(Token_Type::tok_compare_lt, token, tokens, row, col);
  }
  else if (token == ">")
  {
    create_token(Token_Type::tok_compare_gt, token, tokens, row, col);
  }
  else if (token == "||")
  {
    create_token(Token_Type::tok_or, token, tokens, row, col);
  }
  else if (token == "&&")
  {
    create_token(Token_Type::tok_and, token, tokens, row, col);
  }
  else if (token == "->")
  {
    create_token(Token_Type::tok_arrow, token, tokens, row, col);
  }
  else if (token == "i64")
  {
    create_token(Token_Type::tok_i64, token, tokens, row, col);
  }
  else if (token == "i32")
  {
    create_token(Token_Type::tok_i32, token, tokens, row, col);
  }
  else if (token == "i16")
  {
    create_token(Token_Type::tok_i16, token, tokens, row, col);
  }
  else if (token == "i8")
  {
    create_token(Token_Type::tok_i8, token, tokens, row, col);
  }
  else if (token == "bool")
  {
    create_token(Token_Type::tok_bool, token, tokens, row, col);
  }
  else if (token == "float")
  {
    create_token(Token_Type::tok_float, token, tokens, row, col);
  }
  else if (token == "double")
  {
    create_token(Token_Type::tok_double, token, tokens, row, col);
  }
  else if (token == "string")
  {
    create_token(Token_Type::tok_string, token, tokens, row, col);
  }
  else if (token == "object")
  {
    create_token(Token_Type::tok_object, token, tokens, row, col);
  }
  else
  {
    if (is_string)
    {
      create_token(Token_Type::tok_string_lit, token, tokens, row, col);
    }
    else if (is_number(token))
    {
      create_token(Token_Type::tok_number, token, tokens, row, col);
    }
    else
    {
      create_token(Token_Type::tok_identifier, token, tokens, row, col);
    }
  }
}

void handle_single_char_token(char c, vector<shared_ptr<Token>> &tokens, int &row, int &col)
{
  auto str = std::string(1, c);
  ++col;
  switch (c)
  {
  case '(':
    create_token(Token_Type::tok_open_paren, str, tokens, row, col);
    break;
  case ')':
    create_token(Token_Type::tok_close_paren, str, tokens, row, col);
    break;
  case '{':
    create_token(Token_Type::tok_open_curly_bracket, str, tokens, row, col);
    break;
  case '}':
    create_token(Token_Type::tok_close_curly_bracket, str, tokens, row, col);
    break;
  case ';':
    create_token(Token_Type::tok_semicolon, str, tokens, row, col);
    break;
  case ':':
    create_token(Token_Type::tok_colon, str, tokens, row, col);
    break;
  case ',':
    create_token(Token_Type::tok_comma, str, tokens, row, col);
    break;
  case '.':
    create_token(Token_Type::tok_period, str, tokens, row, col);
    break;
  case '=':
    create_token(Token_Type::tok_eq, str, tokens, row, col);
    break;
  case '*':
    create_token(Token_Type::tok_asterisk, str, tokens, row, col);
    break;
  case '/':
    create_token(Token_Type::tok_slash, str, tokens, row, col);
    break;
  case '+':
    create_token(Token_Type::tok_plus, str, tokens, row, col);
    break;
  case '-':
    create_token(Token_Type::tok_minus, str, tokens, row, col);
    break;
  default:
    break;
  }

  add_next_char = false;
  --col;
}

void create_token(Token_Type type, std::string &token, std::vector<shared_ptr<Token>> &tokens, int &row, int &col)
{
  auto tok = std::make_shared<Token>();
  tok->type = type;
  tok->value = token;
  tok->row = row;
  tok->col = col - token.size();

  tokens.push_back(tok);
  token.clear();
}

void print_tokens(vector<shared_ptr<Token>> tokens)
{
  for (auto &token : tokens)
  {
    cout << "[ " << token->type << " : " << token->value << " ] -- " << token->row << ", " << token->col << endl;
  }
}