#include "lexer.h"

vector<shared_ptr<Token>> tokenize(vector<string> content)
{
  vector<shared_ptr<Token>> tokens;
  string token;

  for (auto &line : content)
  {
    for (auto &c : line)
    {
      if (c != ' ' && c != '\t' && c != '\n')
        token += c;

      switch (c)
      {
      case ' ':
        add_token(token, tokens);
        break;
      case '(':
        add_token(token, tokens, true, c);
        break;
      case ')':
        add_token(token, tokens, true, c);
        break;
      case '{':
        add_token(token, tokens, true, c);
        break;
      case '}':
        add_token(token, tokens, true, c);
        break;
      case ';':
        add_token(token, tokens, true, c);
        break;
      case '*':
        add_token(token, tokens, true, c);
        break;
      case '/':
        add_token(token, tokens, true, c);
        break;
      case '+':
        add_token(token, tokens, true, c);
        break;
      case '-':
        if (line[col + 1] != '>')
          add_token(token, tokens, true, c);
        break;
      case '&':
        add_token(token, tokens, true, c);
        break;
      default:
        break;
      }
      col++;
    }
    row++;
    col = 0;
  }

  auto tok = std::make_shared<Token>();
  tok->type = Token_Type::tok_eof;
  tok->row = -1;
  tok->col = -1;
  tok->value = "EOF";
  tokens.push_back(tok);

  return tokens;
}

void add_token(string &token, vector<shared_ptr<Token>> &tokens, bool is_single_char_token, char single_char_token)
{
  string::iterator end_pos = std::remove(token.begin(), token.end(), ' ');
  token.erase(end_pos, token.end());
  if (token.size() < 1)
    return;

  if (token.size() == 1)
  {
    tokens.push_back(create_token(token));
    token.clear();
    return;
  }

  if (is_single_char_token)
    token = token.substr(0, token.size() - 1);

  tokens.push_back(create_token(token));
  token.clear();

  if (is_single_char_token)
    tokens.push_back(create_token(std::string(1, single_char_token)));
}

shared_ptr<Token> create_token(string token)
{
  auto tok = std::make_shared<Token>();
  tok->row = row + 1;
  if (token.size() > 1)
    tok->col = col + 1 - token.size();
  else
    tok->col = col + 1;
  tok->value = token;
  tok->type = get_token_type(token);
  return std::move(tok);
}

Token_Type get_token_type(string token)
{
  if (token == "if")
    return Token_Type::tok_if;
  else if (token == "for")
    return Token_Type::tok_for;
  else if (token == "fn")
    return Token_Type::tok_fn;
  else if (token == "return")
    return Token_Type::tok_return;
  else if (token == "import")
    return Token_Type::tok_import;
  else if (token == "(")
    return Token_Type::tok_open_paren;
  else if (token == ")")
    return Token_Type::tok_close_paren;
  else if (token == "{")
    return Token_Type::tok_open_curly_bracket;
  else if (token == "}")
    return Token_Type::tok_close_curly_bracket;
  else if (token == ";")
    return Token_Type::tok_semicolon;
  else if (token == "*")
    return Token_Type::tok_asterisk;
  else if (token == "/")
    return Token_Type::tok_slash;
  else if (token == "+")
    return Token_Type::tok_plus;
  else if (token == "-")
    return Token_Type::tok_minus;
  else if (token == "&")
    return Token_Type::tok_ampersand;
  else if (token == "=")
    return Token_Type::tok_eq;
  else if (token == "==")
    return Token_Type::tok_compare_eq;
  else if (token == "<")
    return Token_Type::tok_compare_lt;
  else if (token == ">")
    return Token_Type::tok_compare_gt;
  else if (token == "||")
    return Token_Type::tok_or;
  else if (token == "&&")
    return Token_Type::tok_and;
  else if (token == "->")
    return Token_Type::tok_arrow;
  else if (token == "i64")
    return Token_Type::tok_i64;
  else if (token == "i32")
    return Token_Type::tok_i32;
  else if (token == "i16")
    return Token_Type::tok_i16;
  else if (token == "i8")
    return Token_Type::tok_i8;
  else if (token == "bool")
    return Token_Type::tok_bool;
  else if (token == "float")
    return Token_Type::tok_float;
  else if (token == "double")
    return Token_Type::tok_double;
  else if (token == "string")
    return Token_Type::tok_string;
  else if (token == "object")
    return Token_Type::tok_object;
  else
  {
    // if (is_string)
    // {
    // return Token_Type::tok_string_lit, token, tokens);
    // }
    if (is_number(token))
    {
      return Token_Type::tok_number;
    }
    else
    {
      return Token_Type::tok_identifier;
    }
  }
}

bool is_floating_point(const char *str)
{
  char *endptr = 0;
  strtod(str, &endptr);

  if (*endptr != '\0' || endptr == str)
    return false;
  return true;
}

bool is_number(const string &token)
{
  if (is_floating_point(token.c_str()))
    return true;
  return !token.empty() && std::find_if(token.begin(),
                                        token.end(), [](char c) { return !std::isdigit(c); }) == token.end();
}

void print_tokens(const Tokens &tokens)
{
  for (auto &token : tokens)
  {
    cout << "[ " << token->type << " : " << token->value << " ] -- " << token->row << ", " << token->col << endl;
  }
}
