#ifndef PARSER_H
#define PARSER_H

#include <iostream>
#include <vector>
#include "lexer.h"
#include "variables.h"

namespace Parser
{
typedef enum Node_Types
{
  var
} Node_Types;

typedef struct Node
{
  int type;
  Variables::Variable variable;
  int skip;
} Node;

void print_nodes(std::vector<Parser::Node> nodes);
std::vector<Parser::Node> parse_tokens(std::vector<Lexer::Token> tokens);
Parser::Node parse_token(std::vector<Lexer::Token> tokens, int i);
Parser::Node create_int_node(std::vector<Lexer::Token> tokens, int i);
} // namespace Parser

std::ostream &operator<<(std::ostream &os, const Parser::Node &node);

#endif