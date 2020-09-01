#include "parser.h"

std::vector<std::unique_ptr<Node>> parse_tokens(std::vector<std::shared_ptr<Token>> toks)
{
  bin_op_precedence["<"] = 10;
  bin_op_precedence["+"] = 20;
  bin_op_precedence["-"] = 30;
  bin_op_precedence["*"] = 40;

  tokens = toks;
  for (auto &token : toks)
  {
    switch (token->type)
    {
    case Token_Types::kw:
    {
      if (token->value == "let")
      {

        // unique_ptr<Node> variable_declaration_node = variable_declaration();

        // cout << variable_declaration_node->type << endl;
      }
      break;
    }
    default:
      break;
    }
  }

  std::vector<std::unique_ptr<Node>> nodes;

  return nodes;
}

void consume_token()
{
  pos++;
}

int get_precedence()
{
  int prec = bin_op_precedence[tokens[pos]->value];
  if (prec <= 0)
    return -1;
  return prec;
}

std::unique_ptr<Expression_Node> parse_number_expression()
{
  std::unique_ptr<Expression_Node> result = std::make_unique<Number_Expression_Node>(std::stod(tokens[pos]->value));
  consume_token();
  return result;
}

std::unique_ptr<Expression_Node> parse_paren_expression()
{
  consume_token();
  std::unique_ptr<Expression_Node> v = parse_expression();
  if (!v)
    return 0;

  if (tokens[pos]->value != ")")
  {
    cout << "Expected ')'" << endl;
    return 0;
  }

  consume_token();
  return v;
}

std::unique_ptr<Expression_Node> parse_identifier_expression()
{
  std::string name = tokens[pos]->value;

  consume_token();

  if (tokens[pos]->value != "(")
    return std::make_unique<Variable_Declaration_Node>(name);

  consume_token();
  std::vector<std::unique_ptr<Expression_Node>> args;
  if (tokens[pos]->value != ")")
  {
    while (1)
    {
      std::unique_ptr<Expression_Node> arg = parse_expression();
      if (!arg)
        return 0;
      args.push_back(arg);

      if (tokens[pos]->value == ")")
        break;

      if (tokens[pos]->value != ",")
      {
        cout << "Expected ')' or ',' in argument list" << endl;
        return 0;
      }

      consume_token();
    }
  }

  consume_token();

  std::unique_ptr<Call_Node> call_node = std::make_unique<Call_Node>(name, args);
  return call_node;
}

std::unique_ptr<Expression_Node> parse_primary()
{
  std::shared_ptr<Token> token = tokens[pos];
  switch (token->type)
  {
  case Token_Types::id:
  {
    return parse_identifier_expression();
  }
  case Token_Types::lit:
  {
    return parse_number_expression();
  }
  case Token_Types::sep:
  {
    return parse_paren_expression();
  }
  default:
    break;
  }
}

std::unique_ptr<Expression_Node> parse_expression()
{
  std::unique_ptr<Expression_Node> lhs = parse_primary();
  if (!lhs)
    return 0;

  return parse_bin_op_rhs(0, std::move(lhs));
}

std::unique_ptr<Expression_Node> parse_bin_op_rhs(int expr_prec, std::unique_ptr<Expression_Node> lhs)
{
  while (1)
  {
    int tok_prec = get_precedence();

    if (tok_prec < expr_prec)
      return lhs;

    std::string bin_op = tokens[pos]->value;

    consume_token();

    std::unique_ptr<Expression_Node> rhs = parse_primary();
    if (!rhs)
      return 0;

    int next_prec = get_precedence();
    if (tok_prec < next_prec)
    {
      rhs = parse_bin_op_rhs(tok_prec + 1, std::move(rhs));
      if (rhs == 0)
        return 0;
    }

    lhs = std::make_unique<Binary_Expression_Node>(bin_op, lhs, rhs);
  }
}

std::unique_ptr<Expression_Node> parse_prototype()
{
  std::string name = tokens[pos]->value;
  consume_token();

  if (tokens[pos]->value != "(")
  {
    cout << "Expected '(' in function prototype" << endl;
    return 0;
  }

  std::vector<std::string> arg_names;

  std::string tok = tokens[pos]->value;
  while (1)
  {
    if (tok != ")")
    {
      arg_names.push_back(tok);
    }
    consume_token();
  };

  return std::make_unique<Prototype_Node>(name, arg_names);
}

std::unique_ptr<Expression_Node> parse_definition()
{
  consume_token();

  std::unique_ptr<Expression_Node> prototype = parse_prototype();
  if (prototype == 0)
    return 0;

  if (std::unique_ptr<Expression_Node> e = parse_expression())
  {
    return std::make_unique<Function_Node>(prototype, e);
  }
}

// int match_string(std::string match)
// {
//   if (match == tokens[pos]->value)
//   {
//     pos++;
//     return 1;
//   }
//   std::cerr << "Error -- Expected " << match << " found " << tokens[pos]->value << " instead" << endl;
//   return -1;
// }

// RegexMatch *match_regex(std::regex match)
// {
//   if (std::regex_match(tokens[pos]->value, match))
//   {
//     pos++;
//     RegexMatch *match = new RegexMatch();
//     match->result = tokens[pos]->value;
//     match->error = false;
//     return match;
//   }

//   RegexMatch *_match = new RegexMatch();
//   _match->result = tokens[pos]->value;
//   _match->error = true;
//   return _match;
// }

// unique_ptr<Variable_Declaration_Node> variable_declaration()
// {
// }

// std::unique_ptr<Node> variable_declaration()
// {
//   unique_ptr<Node> node = std::make_unique<Node>();
//   int let, colon, equal;
//   RegexMatch *name, *type;

//   if (!(let = match_string("let")))
//   {
//     node->type = NodeTypes::Error;
//     return node;
//   }

//   std::regex name_regex("[a-zA-Z_]+");
//   name = match_regex(name_regex);
//   if (name->error)
//   {
//     node->type = NodeTypes::Error;
//     return node;
//   }

//   if (!(colon = match_string(":")))
//   {
//     node->type = NodeTypes::Error;
//     return node;
//   }

//   std::regex type_regex("int|float");
//   type = match_regex(type_regex);
//   if (name->error)
//   {
//     node->type = NodeTypes::Error;
//     return node;
//   }

//   if (!(equal = match_string("=")))
//   {
//     node->type = NodeTypes::Error;
//     return node;
//   }

//   std::unique_ptr<Expression_Node> expression_node = parse_expression();

//   node->type = NodeTypes::VariableDeclarationNode;

//   return node;
// }

// std::unique_ptr<Expression_Node> parse_expression()
// {
//   unique_ptr<Expression_Node> expression_node = std::make_unique<Expression_Node>();
//   std::string val = tokens[pos]->value;
//   if (is_number(val))
//   {
//     std::unique_ptr<Number_Expression_Node> number_expression = parse_number_expression();

//     RegexMatch *op;

//     // std::regex op_regex("");
//     cout << tokens[pos]->value << endl;
//     std::regex op_regex("[\+|\-|\/|\*]");
//     op = match_regex(op_regex);

//     // cout << "Err? " << op->error << endl;

//     return number_expression;
//   }
//   else if (val == "(")
//   {
//   }
//   else
//   {
//     // parse_identifier_expression();
//   }
// }

// std::unique_ptr<Number_Expression_Node> parse_number_expression()
// {
//   auto result = std::make_unique<Number_Expression_Node>(std::stod(tokens[pos]->value));
//   pos++;

//   return std::move(result);
// }