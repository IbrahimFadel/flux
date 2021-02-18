#include "parser.h"

std::vector<unique_ptr<Node>> parse_tokens(const Tokens &tokens)
{
    toks = tokens;
    cur_tok_index = 0;
    cur_tok = toks[cur_tok_index];

    binop_precedence["="] = 2;
    binop_precedence["<"] = 10;
    binop_precedence[">"] = 10;
    binop_precedence["=="] = 10;
    binop_precedence["!="] = 10;
    binop_precedence["+"] = 20;
    binop_precedence["-"] = 20;
    binop_precedence["*"] = 40;
    binop_precedence["."] = 50;
    binop_precedence["->"] = 50;

    std::vector<unique_ptr<Node>> nodes;

    while (cur_tok->type != Token_Type::tok_eof)
    {
        auto node = parse_token(cur_tok);
        nodes.push_back(std::move(node));
    }

    return nodes;
}

unique_ptr<Node> parse_token(const shared_ptr<Token> &token)
{
    switch (token->type)
    {
    case Token_Type::tok_fn:
        return parse_function_declaration();
    case Token_Type::tok_i64:
        return parse_variable_declaration();
    case Token_Type::tok_i32:
        return parse_variable_declaration();
    case Token_Type::tok_i16:
        return parse_variable_declaration();
    case Token_Type::tok_i8:
        return parse_variable_declaration();
    case Token_Type::tok_bool:
        return parse_variable_declaration();
    case Token_Type::tok_float:
        return parse_variable_declaration();
    case Token_Type::tok_double:
        return parse_variable_declaration();
    case Token_Type::tok_struct:
        return parse_struct_type_declaration();
    case Token_Type::tok_if:
        return parse_if_statement();
    case Token_Type::tok_return:
        return parse_return_statement();
    case Token_Type::tok_import:
        return parse_import_statement();
    case Token_Type::tok_identifier:
    {
        if (std::find(struct_types.begin(), struct_types.end(), cur_tok->value) != struct_types.end())
            return parse_variable_declaration(true);
        return parse_expression();
    }
    default:
        break;
    }
    return nullptr;
}

unique_ptr<Import_Statement> parse_import_statement()
{
    get_next_token(); //? eat 'import'
    throw_if_cur_tok_not_type(Token_Type::tok_string_lit, "Expected path to module in import statement", cur_tok->row, cur_tok->col);
    std::string path = cur_tok->value;
    get_next_token(); //? eat path
    throw_if_cur_tok_not_type(Token_Type::tok_semicolon, "Expected ';' at end of import statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ';
    return std::make_unique<Import_Statement>(path);
}

unique_ptr<Return_Statement> parse_return_statement()
{
    get_next_token(); //? eat 'return'
    auto value = parse_expression();
    return std::make_unique<Return_Statement>(std::move(value));
}

unique_ptr<If_Statement> parse_if_statement()
{
    get_next_token(); //? eat 'if'

    throw_if_cur_tok_not_type(Token_Type::tok_open_paren, "Expected '(' in if statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '(

    std::vector<unique_ptr<Expression>> conditions;
    std::vector<Token_Type> condition_separators;
    while (cur_tok->type != Token_Type::tok_close_paren)
    {
        auto expr = parse_expression(false);
        conditions.push_back(std::move(expr));

        if (cur_tok->type != Token_Type::tok_close_paren)
        {
            condition_separators.push_back(cur_tok->type);
            get_next_token(); //? eat condition separator
        }
    }

    get_next_token(); //? eat ')'

    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{' in if statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    auto then = parse_code_block();

    throw_if_cur_tok_not_type(Token_Type::tok_close_curly_bracket, "Expected '}' in if statement", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '}'

    return std::make_unique<If_Statement>(std::move(conditions), condition_separators, std::move(then));
}

unique_ptr<Expression> parse_struct_type_declaration()
{
    get_next_token(); //? eat 'struct'

    throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected identifier following object token", cur_tok->row, cur_tok->col);
    std::string name = cur_tok->value;
    get_next_token(); //? eat name

    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{' following object token", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    std::map<std::string, std::string> properties;
    std::vector<std::string> property_insetion_order;
    while (cur_tok->type != Token_Type::tok_close_curly_bracket)
    {
        std::string type = get_type();

        throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected identifier in object property", cur_tok->row, cur_tok->col);
        std::string prop_name = cur_tok->value;
        get_next_token(); //? eat name

        throw_if_cur_tok_not_type(Token_Type::tok_semicolon, "Expected ';' in object property", cur_tok->row, cur_tok->col);
        get_next_token(); //? eat ';'
        properties[prop_name] = type;
        property_insetion_order.push_back(prop_name);
    }

    get_next_token(); //? eat '}'
    throw_if_cur_tok_not_type(Token_Type::tok_semicolon, "Expected ';' in object type definition", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ';'

    struct_types.push_back(name);

    return std::make_unique<Struct_Type_Expression>(name, properties, property_insetion_order);
}

unique_ptr<Variable_Declaration> parse_variable_declaration(bool is_struct)
{
    if (is_struct)
    {
        return parse_struct_var_declaration();
    }
    std::string type = get_type();

    throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected identifier following variable type", cur_tok->row, cur_tok->col);
    std::string name = cur_tok->value;
    get_next_token(); //? eat name

    if (cur_tok->type == Token_Type::tok_semicolon)
    {
        get_next_token(); //? eat ';'
        return std::make_unique<Variable_Declaration>(name, type, nullptr);
    }

    throw_if_cur_tok_not_type(Token_Type::tok_eq, "Expected '=' following variable name", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '='

    auto value = parse_expression();

    return std::make_unique<Variable_Declaration>(name, type, std::move(value));
}

static unique_ptr<Variable_Declaration> parse_struct_var_declaration()
{
    std::string type_name = get_type();

    throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected identifier in struct variable declaration", cur_tok->row, cur_tok->col);
    std::string name = cur_tok->value;
    get_next_token(); //? eat name

    if (cur_tok->type == Token_Type::tok_semicolon)
    {
        get_next_token(); //? eat ';'
        return std::make_unique<Variable_Declaration>(name, type_name, nullptr, true);
    }

    throw_if_cur_tok_not_type(Token_Type::tok_eq, "Expected '=' in struct variable declaration", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '='

    auto value = parse_expression();

    return std::make_unique<Variable_Declaration>(name, type_name, std::move(value), true);
}

unique_ptr<Function_Declaration> parse_function_declaration()
{
    get_next_token(); //? eat 'fn'

    throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected identifier following 'fn'", cur_tok->row, cur_tok->col);
    std::string name = cur_tok->value;
    get_next_token(); //? eat name

    throw_if_cur_tok_not_type(Token_Type::tok_open_paren, "Expected '(' in function declaration", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '('

    int i = 0;
    std::map<std::string, std::string> params; //? Param Name, Param Type
    std::string current_param_type = "";       //? Since the user should be able to have n number of pointers, for example, I can't just have an enum of variable types like before. Instead, I'll store types as strings in parser stage then codegen properly. Atleast -- that's the plan for now
    std::string current_param_name = "";
    //? While there are parameters to parse
    while (cur_tok->type != Token_Type::tok_close_paren)
    {
        if (i == 0)
        {
            current_param_type = get_type();
        }
        else if (i == 1)
        {
            current_param_name = cur_tok->value;
            get_next_token(); //? eat param name
            params[current_param_name] = current_param_type;
            current_param_type.clear();
            current_param_name.clear();
        }
        else if (i == 2)
        {
            throw_if_cur_tok_not_type(Token_Type::tok_comma, "Expected ',' in parameter list", cur_tok->row, cur_tok->col);
            get_next_token();
            i = -1; //? This looks pretty terrible, but since i do a ++ at the end of every iteration and it needs to be 0 at the start... it's -1 here
        }
        i++;
    }

    get_next_token(); //? eat ')'

    throw_if_cur_tok_not_type(Token_Type::tok_arrow, "Expected '->' after parameter list", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '->'

    std::string return_type = get_type();

    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{' after function return type", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    auto then = parse_code_block();

    throw_if_cur_tok_not_type(Token_Type::tok_close_curly_bracket, "Expected '}' after function code block", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '}'

    return std::make_unique<Function_Declaration>(name, params, return_type, std::move(then));
}

unique_ptr<Code_Block> parse_code_block()
{
    std::vector<std::unique_ptr<Node>> nodes;

    while (cur_tok->type != Token_Type::tok_close_curly_bracket)
    {
        auto node = parse_token(std::move(cur_tok));
        nodes.push_back(std::move(node));
    }

    return std::make_unique<Code_Block>(std::move(nodes));
}

unique_ptr<Expression> parse_expression(bool needs_semicolon)
{
    auto lhs = parse_primary();
    if (!lhs)
        error("Error parsing primary", cur_tok->row, cur_tok->col);

    auto binop_node = parse_binop_rhs(0, std::move(lhs));

    if (cur_tok->type == Token_Type::tok_open_square_bracket)
    {
        auto expr = parse_square_bracket_expression(std::move(binop_node));
        binop_node = parse_binop_rhs(0, std::move(expr));
    }

    if (needs_semicolon)
    {
        throw_if_cur_tok_not_type(Token_Type::tok_semicolon, "Expected ';' at end of expression", cur_tok->row, cur_tok->col);
        get_next_token(); //? eat ';'
    }
    return binop_node;
}

unique_ptr<Expression> parse_primary()
{
    switch (cur_tok->type)
    {
    case Token_Type::tok_number:
        return parse_number_expression();
    case Token_Type::tok_identifier:
        return parse_identifier_expression();
    case Token_Type::tok_open_paren:
        return parse_paren_expression();
    case Token_Type::tok_asterisk:
        return parse_unary_prefix_operation_expression();
    case Token_Type::tok_ampersand:
        return parse_unary_prefix_operation_expression();
    case Token_Type::tok_open_curly_bracket:
        return parse_struct_value_expression();
    case Token_Type::tok_nullptr:
        get_next_token(); //? eat 'nullptr'
        return std::make_unique<Nullptr_Expression>();
    }
    return nullptr;
}

unique_ptr<Expression> parse_struct_value_expression()
{
    throw_if_cur_tok_not_type(Token_Type::tok_open_curly_bracket, "Expected '{' in struct variable declaration", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat '{'

    std::map<std::string, unique_ptr<Expression>> props;
    std::vector<std::string> property_insetion_order;
    while (cur_tok->type != Token_Type::tok_close_curly_bracket)
    {
        throw_if_cur_tok_not_type(Token_Type::tok_identifier, "Expected identifier in struct variable declaration", cur_tok->row, cur_tok->col);
        std::string property_name = cur_tok->value;
        get_next_token(); //? eat property name

        throw_if_cur_tok_not_type(Token_Type::tok_colon, "Expected ':' in struct variable declaration", cur_tok->row, cur_tok->col);
        get_next_token(); //? eat ':'

        auto val = parse_expression();
        props[property_name] = std::move(val);
        property_insetion_order.push_back(property_name);
    }

    get_next_token(); //? eat '}'

    return std::make_unique<Struct_Value_Expression>(std::move(props), property_insetion_order);
}

unique_ptr<Expression> parse_unary_prefix_operation_expression()
{
    Token_Type op = cur_tok->type;
    get_next_token(); //? eat operator
    auto expr = parse_expression(false);
    return std::make_unique<Unary_Prefix_Operation_Expression>(op, std::move(expr));
}

unique_ptr<Expression> parse_paren_expression()
{
    get_next_token(); //? eat '('
    auto expr = parse_expression(false);
    throw_if_cur_tok_not_type(Token_Type::tok_close_paren, "Expected ')' at end of parentheses expression", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ')'
    return expr;
}

unique_ptr<Expression> parse_binop_rhs(int expression_precedence, unique_ptr<Expression> lhs)
{
    while (true)
    {
        int tok_precedence = get_token_precedence();
        if (tok_precedence < expression_precedence)
            return lhs;

        Token_Type binop = cur_tok->type;

        get_next_token(); //? eat operator

        auto rhs = parse_primary();
        if (!rhs)
            error("Error parsing binary operator right hand side", toks[cur_tok_index - 1]->row, toks[cur_tok_index - 1]->col);

        int next_precedence = get_token_precedence();
        if (tok_precedence < next_precedence)
        {
            rhs = parse_binop_rhs(tok_precedence + 1, std::move(rhs));
            if (!rhs)
                error("Error parsing binary operator right hand side", toks[cur_tok_index - 1]->row, toks[cur_tok_index - 1]->col);
        }

        lhs = std::make_unique<Binary_Operation_Expression>(binop, std::move(lhs), std::move(rhs));
    }

    return nullptr;
}

unique_ptr<Expression> parse_square_bracket_expression(std::unique_ptr<Expression> expr)
{
    get_next_token(); //? eat '['

    auto index = parse_expression(false);

    throw_if_cur_tok_not_type(Token_Type::tok_close_square_bracket, "Expected ']' at end of index access", cur_tok->row, cur_tok->col);
    get_next_token(); //? eat ']'

    return std::make_unique<Index_Accessed_Expression>(std::move(expr), std::move(index));
}

unique_ptr<Expression> parse_number_expression()
{
    double val = std::stod(cur_tok->value.c_str());
    get_next_token(); //? eat number
    return std::make_unique<Number_Expression>(val);
}

unique_ptr<Expression> parse_identifier_expression()
{
    if (toks[cur_tok_index + 1]->type == Token_Type::tok_open_paren)
    {
        std::string fn_name = cur_tok->value;
        std::vector<unique_ptr<Expression>> params;
        get_next_token(); //? eat function name
        get_next_token(); //? eat '('
        int i = 0;
        while (cur_tok->type != Token_Type::tok_close_paren)
        {
            auto param = parse_expression(false);
            params.push_back(std::move(param));
            if (cur_tok->type == Token_Type::tok_comma)
                get_next_token();
        }

        get_next_token(); //? eat ')'
        return std::make_unique<Function_Call_Expression>(fn_name, std::move(params));
    }
    // else if (toks[cur_tok_index + 1]->type == Token_Type::tok_open_square_bracket)
    // {
    //     //TODO array access
    //     std::string var_name = cur_tok->value;
    //     get_next_token(); //? eat var name
    //     get_next_token(); //? eat '['

    //     auto index = parse_expression(false);

    //     throw_if_cur_tok_not_type(Token_Type::tok_close_square_bracket, "Expected ']' at end of index access", cur_tok->row, cur_tok->col);
    //     get_next_token(); //? eat ']'

    //     return std::make_unique<Index_Accessed_Expression>(var_name, std::move(index));
    // }

    auto expr = std::make_unique<Variable_Reference_Expression>(cur_tok->value);
    get_next_token(); //? eat variable name
    return expr;
}

void get_next_token()
{
    cur_tok_index++;
    cur_tok = toks[cur_tok_index];
}

int get_token_precedence()
{
    int tok_precedence = binop_precedence[cur_tok->value];
    if (tok_precedence <= 0)
        return -1;
    return tok_precedence;
}

void error(const char *msg, int row, int col)
{
    std::cout << msg << " at line " << row << " column " << col << std::endl;
    exit(1);
}

void throw_if_cur_tok_not_type(Token_Type type, const char *msg, int line, int position)
{
    if (cur_tok->type != type)
        error(msg, line, position);
}

std::string get_type()
{
    std::string type = cur_tok->value;
    get_next_token();
    while (cur_tok->type == Token_Type::tok_asterisk || cur_tok->type == Token_Type::tok_ampersand)
    {
        type += cur_tok->value;
        get_next_token();
    }

    return type;
}