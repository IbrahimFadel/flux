#ifndef SSC_LEXER_LEXER_H
#define SSC_LEXER_LEXER_H

#include <memory>
#include <algorithm>
#include <vector>
#include <string>
#include <iostream>

using std::cout;
using std::endl;
using std::unique_ptr;

namespace ssc
{
    enum TokenType
    {
        tokFn,
        tokIf,
        tokFor,
        tokReturn,
        tokImport,
        tokPub,
        tokMut,
        tokWhile,

        tokI64,
        tokU64,
        tokI32,
        tokU32,
        tokI16,
        tokU16,
        tokI8,
        tokU8,
        tokF64,
        tokF32,
        tokBool,
        tokDouble,
        tokString,
        tokVoid,
        tokNullptr,
        tokStruct,
        tokArray,

        tokCompareEq,
        tokCompareNe,
        tokCompareLt,
        tokCompareGt,
        tokCompareGtEq,
        tokCompareLtEq,
        tokAnd,
        tokOr,

        tokColon,
        tokSemicolon,
        tokComma,
        tokPeriod,
        tokOpenParen,
        tokCloseParen,
        tokOpenCurlyBracket,
        tokCloseCurlyBracket,
        tokOpenSquareBracket,
        tokCloseSquareBracket,

        tokEq,
        tokAsterisk,
        tokSlash,
        tokPlus,
        tokMinus,
        tokArrow,
        tokAmpersand,
        tokPlusEq,
        tokMinusEq,

        tokStringLit,
        tokNumberLit,
        tokTrue,
        tokFalse,
        tokIdentifier,

        tokEOF
    };

    enum LexerState
    {
        normal,
        string,
        lineComment,
        blockComment
    };

    struct Token
    {
        int row, col;
        std::string value;
        ssc::TokenType type;
    };

    typedef std::vector<unique_ptr<Token>> Tokens;

    class Lexer
    {
    private:
        LexerState state;
        std::vector<std::string> fileContent;
        std::vector<unique_ptr<Token>> tokens;
        std::string token;
        unsigned row;
        unsigned col;

        // void error(std::string msg);

        void reset();
        bool isNumber(const char *str);
        bool isFloatingPoint(const char *str);
        void addValidLexeme(char c = '\0');
        unique_ptr<Token> constructToken(TokenType type);

    public:
        Tokens tokenize(std::vector<std::string> content);
        void printTokens(const Tokens &tokens);
    };

    // class Lexer
    // {
    // private:
    //     void addToken(std::string &token, vector<uniquePtr<Token>> &tokens, bool isSingleCharToken = false, char singleCharToken = '\0');
    //     uniquePtr<Token> createToken(std::string token);
    //     ssc::TokenType getTokenType(std::string token);

    //     bool isNumber(const std::string &token);
    //     bool isFloatingPoint(const char *str);

    //     int row = 0, col = 0;
    //     bool isStd::string = false;

    // public:
    //     Tokens tokenize(vector<std::string> content);
    //     void printTokens(const Tokens &tokens);
    // };
} // namespace ssc

#endif