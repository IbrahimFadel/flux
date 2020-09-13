#include "parser.h"

void parse_tokens(std::vector<std::shared_ptr<Token>> tokens)
{
    cout << "parsing " << tokens.size() << " tokens" << endl;
    for (auto &token : tokens)
    {
        auto node = parse_token(token);
    }
}

unique_ptr<Node> parse_token(std::shared_ptr<Token> token)
{
    unique_ptr<Node> node = std::make_unique<Node>();

    switch (token->type)
    {
    case Token_Types::tok_fn:
    {
        cout << "Found fn" << endl;
        break;
    }
    default:
        break;
    }

    return node;
}