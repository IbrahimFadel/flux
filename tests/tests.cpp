#include <gtest/gtest.h>
#include <common.h>
#include <lexer.h>

TEST(LexerTests, SingleCharOperators)
{
    auto file_content = get_file_content("../tests/lexer_test.ss");
    auto tokens = tokenize(file_content);

    vector<Token_Type> token_types = {Token_Type::tok_fn, Token_Type::tok_identifier, Token_Type::tok_open_paren, Token_Type::tok_close_paren, Token_Type::tok_arrow, Token_Type::tok_i32, Token_Type::tok_open_curly_bracket, Token_Type::tok_return, Token_Type::tok_number, Token_Type::tok_semicolon, Token_Type::tok_close_curly_bracket, Token_Type::tok_semicolon, Token_Type::tok_eof};

    for (int i = 0; i < tokens.size(); i++)
    {
        EXPECT_EQ(tokens[i]->type, token_types[i]);
    }
}

int main(int argc, char **argv)
{
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}