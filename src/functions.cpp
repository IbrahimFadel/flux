#include "functions.h"

std::vector<std::string> Functions::get_fn_parameters(std::vector<Lexer::Token> tokens, int i)
{
  std::vector<std::string> params;
  int counter = 0;
  for (int x = i; x < tokens.size(); x++)
  {
    if (tokens[x].value == ")" && tokens[x].type == Lexer::Token_Types::sep)
    {
      return params;
    }
    if (counter % 2 == 0)
    {
      params.push_back(tokens[x].value);
    }
    counter++;
  }
  return params;
}

int Functions::get_fn_return_type(std::vector<Lexer::Token> tokens, int i)
{
  std::string type = tokens[i].value;
  if (type == "void")
  {
    return Variables::Variable_Types::_void;
  }
  else if (type == "int")
  {
    return Variables::Variable_Types::integer;
  }
  return Variables::Variable_Types::_void;
}

Parser::Then Functions::get_fn_then(std::vector<Lexer::Token> tokens, int i)
{
  Parser::Then then;
  int open_curly_brackets = 1;
  int closed_curly_brackets = 0;
  for (int x = i; x < tokens.size(); x++)
  {
    if (open_curly_brackets == closed_curly_brackets)
    {
      break;
    }
    if (tokens[x].type == Lexer::Token_Types::sep && tokens[x].value == "{")
    {
      open_curly_brackets++;
    }
    else if (tokens[x].type == Lexer::Token_Types::sep && tokens[x].value == "}")
    {
      closed_curly_brackets++;
    }
    then.tokens.push_back(tokens[x]);
  }
  return then;
}

Parser::Function *Functions::get_fn(std::string name)
{
  Parser::Function *function = new Parser::Function();
  for (int i = 0; i < Functions::functions.size(); i++)
  {
    if (Functions::functions[i]->name == name)
    {
      return Functions::functions[i];
    }
  }

  return function;
}