#include "parser.h"

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
    for (auto &token : tokens)
    {
        unique_ptr<Node> node = parse_token(token);
    }
}

unique_ptr<Node> parse_token(std::shared_ptr<Token> token)
{
    unique_ptr<Node> node = std::make_unique<Node>();

    // switch (token->type)
    // {
    // case Token_Types::kw:
    // {
    //     if(token->value)
    //     break;
    // }
    // default:
    //     break;
    // }

    return node;
}