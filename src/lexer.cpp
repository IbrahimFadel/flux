#include "lexer.h"

bool is_seperator(std::string token)
{
  return std::find(seperators.begin(), seperators.end(), token) != seperators.end();
}

bool is_type(std::string token)
{
  return std::find(types.begin(), types.end(), token) != types.end();
}

bool is_number(std::string string)
{
  std::string acceptable_chars = "-0123456789.";
  return (string.find_first_not_of(acceptable_chars.substr(0, 13)) == std::string::npos);
}

bool is_literal(std::string token, int lexer_state)
{
  if (lexer_state != Lexer_State::string)
  {
    return is_number(token);
  }
  return lexer_state == Lexer_State::string;
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

void add_token(std::string &token, std::vector<std::shared_ptr<Token>> &tokens, int row, int column, int lexer_state)
{
  std::shared_ptr<Token> tok;
  if (is_keyword(token))
  {
    tok = create_token(token, Token_Types::kw, row, column - token.size(), column);
  }
  else if (is_operator(token))
  {
    tok = create_token(token, Token_Types::op, row, column - token.size(), column);
  }
  else if (is_literal(token, lexer_state))
  {
    tok = create_token(token, Token_Types::lit, row, column - token.size(), column);
  }
  else if (is_seperator(token))
  {
    tok = create_token(token, Token_Types::sep, row, column - token.size(), column);
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
  int row = 0;
  int column = 0;

  std::vector<std::shared_ptr<Token>> tokens;
  std::string token = "";

  int lexer_state = Lexer_State::normal;
  int string_state = String_State::none;

  int i = 0;
  for (char const &c : buffer)
  {
    switch (c)
    {
    case '{':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      std::shared_ptr<Token> tok = create_token("{", Token_Types::sep, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case '}':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (token.length() > 0)
      {
        add_token(token, tokens, row, column, lexer_state);
      }
      std::shared_ptr<Token> tok = create_token("}", Token_Types::sep, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case '(':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (token.length() > 0)
      {
        add_token(token, tokens, row, column, lexer_state);
      }
      std::shared_ptr<Token> tok = create_token("(", Token_Types::sep, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case ')':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (token.length() > 0)
      {
        add_token(token, tokens, row, column, lexer_state);
      }
      std::shared_ptr<Token> tok = create_token(")", Token_Types::sep, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case ',':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      add_token(token, tokens, row, column, lexer_state);
      std::shared_ptr<Token> tok = create_token(",", Token_Types::eol, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case '[':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      std::shared_ptr<Token> tok = create_token("[", Token_Types::sep, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case ']':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      add_token(token, tokens, row, column, lexer_state);
      std::shared_ptr<Token> tok = create_token("]", Token_Types::sep, row, column - token.length(), column);
      tokens.push_back(tok);

      token.clear();
      break;
    }
    case '<':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (token == "array")
      {
        std::shared_ptr<Token> tok = create_token(token, Token_Types::kw, row, column - token.length(), column);
        tokens.push_back(tok);

        token.clear();
        tok = create_token("<", Token_Types::sep, row, column - token.length(), column);
        tokens.push_back(tok);
      }
      else
      {
        token += c;
        if (i == buffer.length() - 1)
        {
          add_token(token, tokens, row, column, lexer_state);
        }
      }
      break;
    }
    case '>':
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (is_keyword(token) && is_type(token))
      {
        std::shared_ptr<Token> tok = create_token(token, Token_Types::kw, row, column - token.length(), column);
        tokens.push_back(tok);

        token.clear();
        tok = create_token(">", Token_Types::sep, row, column - token.length(), column);
        tokens.push_back(tok);
      }
      else
      {
        token += c;
        if (i == buffer.length() - 1)
        {
          add_token(token, tokens, row, column, lexer_state);
        }
      }
      break;
    case ' ':
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (token.length() > 0)
      {
        add_token(token, tokens, row, column, lexer_state);
      }
      break;
    case '\n':
      row++;
      column = -1;
      break;
    case '\t':
      break;
    case ':':
    {
      if (lexer_state == Lexer_State::string)
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
    case ';':
    {
      if (lexer_state == Lexer_State::string)
      {
        token += c;
        break;
      }
      if (token.size() > 0)
      {
        add_token(token, tokens, row, column, lexer_state);
      }
      token += c;
      std::shared_ptr<Token> tok2 = create_token(token, Token_Types::eol, row, column, column);
      tokens.push_back(std::move(tok2));

      token.clear();
      break;
    }
    case '"':
      if (string_state == String_State::none)
      {
        string_state = String_State::double_quotes;
      }

      if (string_state == String_State::double_quotes)
      {
        if (lexer_state != Lexer_State::string)
        {
          string_state = String_State::double_quotes;
          lexer_state = Lexer_State::string;
        }
        else
        {
          add_token(token, tokens, row, column, lexer_state);
          lexer_state = Lexer_State::normal;
        }
      }
      else
      {
        token += c;
        if (lexer_state != Lexer_State::string)
        {
          string_state = String_State::double_quotes;
          lexer_state = Lexer_State::string;
        }
      }
      break;
    case '\'':
      if (string_state == String_State::none)
      {
        string_state = String_State::single_quotes;
      }

      if (string_state != String_State::double_quotes)
      {
        if (lexer_state != Lexer_State::string)
        {
          string_state == String_State::single_quotes;
          lexer_state = Lexer_State::string;
        }
        else
        {
          add_token(token, tokens, row, column, lexer_state);
          lexer_state = Lexer_State::normal;
        }
      }
      else
      {
        token += c;
        if (lexer_state != Lexer_State::string)
        {
          string_state = String_State::double_quotes;
          lexer_state = Lexer_State::string;
        }
      }
      break;
    default:
      token += c;
      if (i == buffer.length() - 1)
      {
        add_token(token, tokens, row, column, lexer_state);
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
  for (auto &token : tokens)
  {
    cout << "['" << token->value << "' : " << token->type << "] - " << token->start_row << ' ' << token->start_col << endl;
  }
  cout << "-------------" << endl;
}