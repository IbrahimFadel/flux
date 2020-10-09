#include "parser.h"

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
    toks = tokens;
    cur_tok = toks[tok_pointer];

    bin_op_precedence["<"] = 10;
    bin_op_precedence["+"] = 20;
    bin_op_precedence["-"] = 20;
    bin_op_precedence["*"] = 40;

    std::vector<std::unique_ptr<Expression_Node>> nodes;
    std::unique_ptr<Expression_Node> node;
    bool ate_semicolon = false;
    while (cur_tok->type != Token_Types::tok_eof)
    {
        switch (cur_tok->type)
        {
        case Token_Types::tok_fn:
            cout << "Parsing function declaration" << endl;
            node = parse_fn_declaration();
            ate_semicolon = false;
            break;
        case Token_Types::tok_int:
            cout << "Parsing variable declaration" << endl;
            node = parse_variable_declaration();
            ate_semicolon = true;
            break;
        default:
            break;
        }
        nodes.push_back(std::move(node));
        if (!ate_semicolon)
        {
            get_next_token();
        }
    }
}

std::unique_ptr<Variable_Node> parse_variable_declaration()
{
    const char *type_string = cur_tok->value.c_str();

    Variable_Types type = type_string_to_variable_type(type_string);

    get_next_token();

    std::string name = cur_tok->value;

    get_next_token();

    get_next_token();

    auto val = parse_expression();
    if (!val)
    {
        error("Expected expression");
        return nullptr;
    }

    return std::make_unique<Variable_Node>(name, type, std::move(val));
}

std::unique_ptr<Function_Node> parse_fn_declaration()
{
    get_next_token();
    auto proto = parse_prototype();
    if (!proto)
        return nullptr;

    auto expressions = parse_fn_body();
    // auto E = parse_expression();
    // if (auto E = parse_expression())
    // return std::make_unique<Function_Node>(std::move(proto), std::move(E));
    return nullptr;
    // return std::make_unique<Function_Node>(std::move(proto), nullptr);
}

std::unique_ptr<Prototype_Node> parse_prototype()
{
    if (cur_tok->type != Token_Types::tok_identifier)
        return error_p("Expected function name in prototype");

    std::string fn_name = cur_tok->value;

    get_next_token();

    if (cur_tok->type != Token_Types::tok_open_paren)
        return error_p("Expected '(' in prototype");

    get_next_token();
    std::vector<Token_Types> arg_types;
    std::vector<std::string> arg_names;

    int param_counter = 0;
    while (cur_tok->type != Token_Types::tok_close_paren)
    {
        if (param_counter == 0)
        {
            arg_types.push_back(cur_tok->type);
        }
        else if (param_counter == 1)
        {
            arg_names.push_back(cur_tok->value);
        }
        else if (param_counter == 2)
        {
            if (cur_tok->type == Token_Types::tok_comma)
            {
                param_counter = 0;
            }
        }

        get_next_token();
        param_counter++;
    }

    if (cur_tok->type != Token_Types::tok_close_paren)
        return error_p("Expected ')' in prototype");

    get_next_token();

    if (cur_tok->type != Token_Types::tok_arrow)
        return error_p("Expected '->' to indicate return type in prototype");

    get_next_token();

    Token_Types return_type = cur_tok->type;

    get_next_token();

    if (cur_tok->type != Token_Types::tok_open_curly_bracket)
    {
        char *err_msg;
        sprintf(err_msg, "Expected '{' on line %d position %d", cur_tok->row, cur_tok->col);
        return error_p(err_msg);
    }

    get_next_token();

    return std::make_unique<Prototype_Node>(fn_name, arg_types, arg_names, return_type);
}

std::vector<std::unique_ptr<Expression_Node>> parse_fn_body()
{
    std::vector<std::unique_ptr<Expression_Node>> nodes;
    std::unique_ptr<Expression_Node> node;
    bool ate_semicolon = false;
    while (cur_tok->type != Token_Types::tok_close_curly_bracket)
    {
        switch (cur_tok->type)
        {
        case Token_Types::tok_fn:
            cout << "Parsing function declaration" << endl;
            node = parse_fn_declaration();
            ate_semicolon = false;
            break;
        case Token_Types::tok_int:
            cout << "Parsing variable declaration" << endl;
            node = parse_variable_declaration();
            ate_semicolon = true;
            break;
        default:
            break;
        }
        nodes.push_back(std::move(node));
        if (!ate_semicolon)
        {
            get_next_token();
        }
    }

    return nodes;
}

std::unique_ptr<Expression_Node> parse_expression(bool needs_semicolon)
{
    auto lhs = parse_primary();
    if (!lhs)
    {
        error("Error parsing primary");
        return nullptr;
    };
    auto bin_op_rhs = parse_bin_op_rhs(0, std::move(lhs));
    if (needs_semicolon)
        get_next_token();
    return bin_op_rhs;
}

std::unique_ptr<Expression_Node> parse_primary()
{
    switch (cur_tok->type)
    {
    case Token_Types::tok_identifier:
        return parse_identifier_expression();
    case Token_Types::tok_number:
        return parse_number_expression();
    default:
        break;
    }
}

std::unique_ptr<Expression_Node> parse_identifier_expression()
{
    std::string id_name = cur_tok->value;

    get_next_token();

    if (cur_tok->type != Token_Types::tok_open_paren)
        return std::make_unique<Variable_Expression_Node>(id_name);

    get_next_token();

    std::vector<std::unique_ptr<Expression_Node>> args;
    if (cur_tok->type != Token_Types::tok_close_paren)
    {
        while (true)
        {
            if (auto arg = parse_expression(false))
            {
                args.push_back(std::move(arg));
            }
            else
            {
                error("Error parsing function call parameters");
                return nullptr;
            }

            if (cur_tok->type == Token_Types::tok_close_paren)
                break;
            if (cur_tok->type != Token_Types::tok_comma)
                return error("Expected ')' or ',' in argument list");

            get_next_token();
        }
    }

    get_next_token();

    return std::make_unique<Call_Expression_Node>(id_name, std::move(args));
    // return nullptr;
}

std::unique_ptr<Expression_Node> parse_number_expression()
{
    auto number_expression = std::make_unique<Number_Expression_Node>(std::stod(cur_tok->value));
    get_next_token();
    return std::move(number_expression);
}

std::unique_ptr<Expression_Node> parse_bin_op_rhs(int expr_prec, std::unique_ptr<Expression_Node> lhs)
{
    while (true)
    {
        int tok_prec = get_tok_precedence();
        // cout << "tok_prec: " << tok_prec << endl;
        // cout << "expr_prec: " << expr_prec << endl;
        // cout << (tok_prec < expr_prec) << endl;
        if (tok_prec < expr_prec)
        {
            // cout << "less" << endl;
            // cout << "less before type: " << cur_tok->type << endl;
            // get_next_token();
            // cout << "less after type: " << cur_tok->type << endl;
            return lhs;
        }

        std::string bin_op = cur_tok->value;
        // cout << "bin_op: " << bin_op << endl;

        get_next_token();

        auto rhs = parse_primary();
        if (!rhs)
        {
            error("Error parsing right hand side");
            return nullptr;
        }

        // cout << "RHS: " << rhs->value << endl;

        int next_prec = get_tok_precedence();
        // cout << "next_prec: " << next_prec << endl;
        if (tok_prec < next_prec)
        {
            rhs = parse_bin_op_rhs(tok_prec + 1, std::move(rhs));
            if (!rhs)
                return nullptr;
        }

        // cout << "end" << endl;

        lhs = std::make_unique<Binary_Expression_Node>(bin_op, std::move(lhs), std::move(rhs));
    }
}

Variable_Types type_string_to_variable_type(const char *str)
{
    if (!strcmp(str, "int"))
    {
        return Variable_Types::type_int;
    }

    return Variable_Types::type_int;
}

void get_next_token()
{
    tok_pointer++;
    cur_tok = toks[tok_pointer];
}

std::unique_ptr<Expression_Node> error(const char *str)
{
    fprintf(stderr, "LogError: %s\n", str);
    return nullptr;
}

std::unique_ptr<Prototype_Node> error_p(const char *str)
{
    error(str);
    return nullptr;
}

int get_tok_precedence()
{
    int tok_prec = bin_op_precedence[cur_tok->value];
    if (tok_prec <= 0)
        return -1;
    return tok_prec;
}