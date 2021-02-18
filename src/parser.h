#ifndef PARSER_H
#define PARSER_H

#include "ast.h"

static int cur_tok_index;
static shared_ptr<Token> cur_tok;
static Tokens toks;
static std::map<std::string, int> binop_precedence;
inline std::vector<std::string> struct_types;

Nodes parse_tokens(const Tokens &token);
static unique_ptr<Node> parse_token(const shared_ptr<Token> &token);

//! Expression Parsing
static unique_ptr<Expression> parse_expression(bool needs_semicolon = true);
static unique_ptr<Expression> parse_primary();
static unique_ptr<Expression> parse_binop_rhs(int expression_precedence, unique_ptr<Expression> lhs);
static unique_ptr<Expression> parse_paren_expression();
static unique_ptr<Expression> parse_number_expression();
static unique_ptr<Expression> parse_identifier_expression();
static unique_ptr<Expression> parse_unary_prefix_operation_expression();
static unique_ptr<Expression> parse_struct_type_declaration();
static unique_ptr<Expression> parse_struct_value_expression();
static unique_ptr<Expression> parse_square_bracket_expression(std::unique_ptr<Expression> expr);

//! Variable Declaration Parsing
static unique_ptr<Variable_Declaration> parse_variable_declaration(bool is_struct = false);
static unique_ptr<Variable_Declaration> parse_struct_var_declaration();

//! Function Parsing
static unique_ptr<Function_Declaration> parse_function_declaration();
static unique_ptr<Code_Block> parse_code_block();

//! Other
static unique_ptr<If_Statement> parse_if_statement();
static unique_ptr<Return_Statement> parse_return_statement();
static unique_ptr<Import_Statement> parse_import_statement();

//! Common
static void get_next_token();
static int get_token_precedence();
static void throw_if_cur_tok_not_type(Token_Type type, const char *msg, int line, int position);
static std::string get_type();
static void error(const char *msg, int row, int col);

// #define UNKOWN_LINE -1
// #define UNKNOWN_COLUMN -1

// #include "ast.h"

// static Tokens toks;
// static shared_ptr<Token> cur_tok;
// static shared_ptr<Token> last_tok;
// static int tok_pointer = 0;
// static std::map<std::string, int> binop_precedence;

// void print_nodes(const Nodes &nodes);
// Nodes parse_tokens(const Tokens &tokens);
// static unique_ptr<Node> parse_token(const shared_ptr<Token> &token);

// static std::vector<std::string> parsed_object_types;

// static unique_ptr<Expression_Node> parse_expression(bool needs_semicolon = true);
// static unique_ptr<Expression_Node> parse_primary();
// static unique_ptr<Expression_Node> parse_binop_rhs(int expression_precedence, unique_ptr<Expression_Node> lhs);
// static unique_ptr<Expression_Node> parse_number_expression();
// static unique_ptr<Expression_Node> parse_identifier_expression();
// static unique_ptr<Function_Node> parse_fn_declaration();
// static unique_ptr<Prototype_Node> parse_fn_prototype();
// static unique_ptr<Then_Node> parse_then();
// static unique_ptr<Variable_Declaration_Node> parse_variable_declaration(bool is_object_type = false);
// static unique_ptr<If_Node> parse_if();
// static std::tuple<std::vector<std::unique_ptr<Condition_Node>>, std::vector<Token_Type>> parse_condition(Token_Type end_token);
// static unique_ptr<Function_Call_Node> parse_function_call_node(std::string name);
// static unique_ptr<Import_Node> parse_import();
// static unique_ptr<For_Node> parse_for();
// static unique_ptr<Object_Node> parse_object_node();
// static unique_ptr<Variable_Declaration_Node> parse_object_variable_declaration();
// static unique_ptr<Variable_Declaration_Node> parse_primitive_type_variable_declaration();
// static unique_ptr<Object_Expression> parse_object_expression();
// static unique_ptr<String_Expression> parse_string_expression();
// static unique_ptr<Return_Node> parse_return();
// static unique_ptr<Expression_Node> parse_ampersand_expression();
// static unique_ptr<Expression_Node> parse_asterisk_expression();
// static unique_ptr<Expression_Node> parse_open_square_bracket_expression();
// static unique_ptr<Object_Property_Assignment_Node> parse_object_property_assignment_node(std::string object_name, bool has_asterisk = false);

// static void throw_if_cur_tok_is_type(Token_Type type, const char *msg, int line, int position);
// static void throw_if_cur_tok_not_type(Token_Type type, const char *msg, int line, int position);
// static Variable_Type token_type_to_variable_type(Token_Type type);
// static Variable_Type variable_type_to_pointer_variable_type(Variable_Type type);
// static Variable_Type variable_type_to_reference_variable_type(Variable_Type type);
// static void get_next_token();
// static int get_token_precedence();
// static void error(const char *arg, int line, int column);

#endif