#include "lexer.h"

using std::cout;
using std::endl;

bool is_number(std::string string)
{
  std::string acceptable_chars = "-0123456789.";
  return (string.find_first_not_of(acceptable_chars.substr(0, 13)) == std::string::npos);
}

bool is_literal(std::string token, bool is_string)
{
  if(!is_string)
  {
    return is_number(token);
  }
  return is_string;
}

bool is_operator(std::string token)
{
  return std::find(operators.begin(), operators.end(), token) != operators.end();
}

bool is_keyword(std::string token)
{
  return std::find(keywords.begin(), keywords.end(), token) != keywords.end();
}

std::shared_ptr<Token> create_token(std::string value, int type, int row, int start_col, int end_col)
{
  std::shared_ptr<Token> tok = std::shared_ptr<Token>(new Token{value, type, row, start_col, end_col});
  return tok;
}

void add_token(std::string &token, std::vector<std::shared_ptr<Token>> &tokens, int row, int column, bool is_string)
{
  std::shared_ptr<Token> tok;
  if(is_keyword(token))
  {
    tok = create_token(token, Token_Types::kw, row, column - token.size(), column);
  }
  else if(is_operator(token))
  {
    tok = create_token(token, Token_Types::op, row, column - token.size(), column);
  }
  else if(is_literal(token, is_string))
  {
    tok = create_token(token, Token_Types::lit, row, column - token.size(), column);
  }
  else
  {
    tok = create_token(token, Token_Types::id, row, column - token.size(), column);
  }

  tokens.push_back(std::move(tok));
  token.clear();
}

std::vector<std::shared_ptr<Token>> get_tokens(const std::string buffer)
{
  int row = -1;
  int column = -1;

  std::vector<std::shared_ptr<Token>> tokens;
  std::string token = "";

  bool is_string = false;
  bool double_quotes = false;

  int i = 0;
  for(char const &c : buffer)
  {
    switch (c)
    {
    case ' ':
      if(is_string)
      {
        token += c;
        break;
      }
      add_token(token, tokens, row, column, is_string);
      break;
    case '\n':
      row++;
      column = -1;
      break;
    case '\t':
      break;
    case ':': {
      if(is_string)
      {
        token += c;
        break;
      }
      std::shared_ptr<Token> tok = create_token(token, Token_Types::id, row, column - token.size(), column);
      tokens.push_back(std::move(tok));
      token.clear();
      token += c;
      break;
    }
    case ';': {
      if(is_string)
      {
        token += c;
        break;
      }
      if(token.size() > 0)
      {
        add_token(token, tokens, row, column, is_string);
      }
      token += c;
      std::shared_ptr<Token> tok2 = create_token(token, Token_Types::eol, row, column, column);
      tokens.push_back(std::move(tok2));
      token.clear();
      break;
    }
    case '"':
      if (double_quotes)
      {
        if(!is_string)
        {
          double_quotes = true;
          is_string = true;
        }
        else
        {
          add_token(token, tokens, row, column, is_string);
          is_string = !is_string;
        }
      }
      else
      {
        if(!is_string)
        {
          double_quotes = true;
          is_string = true;
        }
      }
      break;
    case '\'':
      if (!double_quotes)
      {
        if(!is_string)
        {
          double_quotes = false;
          is_string = true;
        }
        else
        {
          add_token(token, tokens, row, column, is_string);
          is_string = !is_string;
        }
      }
      else
      {
        if(!is_string)
        {
          double_quotes = true;
          is_string = true;
        }
      }
      break;
    default:
      token += c;
      if(i == buffer.length() - 1)
      {
        add_token(token, tokens, row, column, is_string);
      }
      break;
    }
    column++;
    i++;
  }

  return tokens;
}

void print_tokens(std::vector<std::shared_ptr<Token>> &tokens)
{
  cout << "-------------" << endl;
  for(auto& token : tokens)
  {
    cout << "['" << token->value << "' : " << token->type << "]" << endl;
  }
  cout << "-------------" << endl;
}