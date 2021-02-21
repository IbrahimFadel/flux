#ifndef SSC_AST_NODES_H
#define SSC_AST_NODES_H

namespace ssc
{
    enum NodeType
    {

    };

    class Node
    {
    private:
        NodeType type;

    public:
    };

    class Expression : public Node
    {
    private:
    public:
    };

    class NumberExpression : public Expression
    {
    private:
        double value;

    public:
        NumberExpression(double value) : value(value){};
    };

    class BinaryOperationExpression : public Expression
    {
    private:
        unique_ptr<Expression> lhs;
        unique_ptr<Expression> rhs;
        TokenType op;

    public:
        BinaryOperationExpression(unique_ptr<Expression> lhs, unique_ptr<Expression> rhs, TokenType op) : lhs(std::move(lhs)), rhs(std::move(rhs)), op(op){};
    };

    struct Parameter
    {
        bool mut;
        std::string type;
        std::string name;
    };

    class FunctionDeclaration : public Node
    {
    private:
        bool pub;
        std::string name;
        std::vector<Parameter> parameters;
        std::string returnType;
        std::vector<unique_ptr<Node>> then;

    public:
        FunctionDeclaration(bool pub, std::string name, std::vector<Parameter> parameters, std::string returnType, std::vector<unique_ptr<Node>> then) : pub(pub), name(name), parameters(parameters), returnType(returnType), then(std::move(then)){};
    };

    class VariableDeclaration : public Node
    {
    private:
        bool pub;
        bool mut;
        std::string type;
        std::string name;
        unique_ptr<Expression> value;

    public:
        VariableDeclaration(bool pub, bool mut, std::string type, std::string name, unique_ptr<Expression> value) : pub(pub), mut(mut), type(type), name(name), value(std::move(value)){};
    };

    typedef std::vector<unique_ptr<Node>> Nodes;

}; // namespace ssc

    // #include <string.h>
    // #include <memory>
    // #include <vector>
    // #include <map>

    // #include "llvm/IR/Value.h"
    // #include "lexer/lexer.h"

    // using std::unique_ptr;

    // namespace ssc
    // {

    //     class Node
    //     {
    //     private:
    //     public:
    //         virtual llvm::Value *codegen() = 0;
    //     };

    //     class Expression : public Node
    //     {
    //     private:
    //     public:
    //         virtual llvm::Value *codegen() = 0;
    //     };

    //     class NumberExpression : public Expression
    //     {
    //     private:
    //         double value;

    //     public:
    //         NumberExpression(double value) : value(value){};
    //         llvm::Value *codegen();

    //         double getValue();
    //     };

    //     class StringLiteralExpression : public Expression
    //     {
    //     private:
    //         std::string value;

    //     public:
    //         StringLiteralExpression(std::string value) : value(value){};
    //         llvm::Value *codegen();
    //         std::string getValue();
    //     };

    //     class NullptrExpression : public Expression
    //     {
    //     private:
    //     public:
    //         llvm::Value *codegen();
    //     };

    //     class VariableReferenceExpression : public Expression
    //     {
    //     private:
    //         std::string name;

    //     public:
    //         VariableReferenceExpression(std::string name) : name(name){};
    //         llvm::Value *codegen();
    //         std::string getName();
    //     };

    //     class BinaryOperationExpression : public Expression
    //     {
    //     private:
    //         ssc::TokenType op;
    //         unique_ptr<Expression> lhs;
    //         unique_ptr<Expression> rhs;

    //     public:
    //         BinaryOperationExpression(ssc::TokenType op, unique_ptr<Expression> lhs, unique_ptr<Expression> rhs) : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)){};
    //         llvm::Value *codegen();
    //         ssc::TokenType getOp();
    //         unique_ptr<Expression> &getLHS();
    //         unique_ptr<Expression> &getRHS();
    //     };

    //     class IndexAccessedExpression : public Expression
    //     {
    //     private:
    //         unique_ptr<Expression> expr;
    //         unique_ptr<Expression> index;

    //     public:
    //         IndexAccessedExpression(unique_ptr<Expression> expr, unique_ptr<Expression> index) : expr(std::move(expr)), index(std::move(index)){};
    //         llvm::Value *codegen();

    //         Expression *getExpression();
    //         Expression *getIndex();
    //     };

    //     class UnaryPrefixOperationExpression : public Expression
    //     {
    //     private:
    //         ssc::TokenType op;
    //         unique_ptr<Expression> value;

    //     public:
    //         UnaryPrefixOperationExpression(ssc::TokenType op, unique_ptr<Expression> value) : op(op), value(std::move(value)){};
    //         llvm::Value *codegen();

    //         Expression *getValue();
    //         TokenType getOp();
    //     };

    //     class CodeBlock : public Node
    //     {
    //     private:
    //         std::vector<unique_ptr<Node>> nodes;

    //     public:
    //         CodeBlock(std::vector<unique_ptr<Node>> nodes) : nodes(std::move(nodes)){};
    //         llvm::Value *codegen();

    //         std::vector<unique_ptr<Node>> &getNodes();
    //     };

    //     class FunctionDeclaration : public Node
    //     {
    //     private:
    //         std::string name;
    //         std::map<std::string, std::string> params;
    //         std::string returnType;
    //         std::unique_ptr<CodeBlock> then;
    //         std::map<std::string, llvm::Value *> variables;

    //     public:
    //         FunctionDeclaration(std::string name, std::map<std::string, std::string> params, std::string returnType, std::unique_ptr<CodeBlock> then) : name(name), params(params), returnType(returnType), then(std::move(then)){};
    //         llvm::Value *codegen();

    //         void setVariable(std::string name, llvm::Value *v);
    //         llvm::Value *getVariable(std::string name);
    //         std::string getName();
    //         std::map<std::string, std::string> getParams();
    //         std::string getReturnType();
    //         CodeBlock *getThen();
    //     };

    //     class VariableDeclaration : public Node
    //     {
    //     private:
    //         std::string name;
    //         std::string type;
    //         unique_ptr<Expression> value;
    //         bool isStruct = false;

    //     public:
    //         VariableDeclaration(std::string name, std::string type, unique_ptr<Expression> value) : name(name), type(type), value(std::move(value)){};
    //         VariableDeclaration(std::string name, std::string type, unique_ptr<Expression> value, bool isStruct) : name(name), type(type), value(std::move(value)), isStruct(isStruct){};
    //         llvm::Value *codegen();

    //         bool getIsStruct();
    //         std::string getName();
    //         std::string getType();
    //         Expression *getValue();
    //     };

    //     class StructTypeExpression : public Expression
    //     {
    //     private:
    //         std::string name;
    //         std::map<std::string, std::string> properties;
    //         std::vector<std::string> propertyInsertionOrder;

    //     public:
    //         StructTypeExpression(std::string name, std::map<std::string, std::string> properties, std::vector<std::string> propertyInsertionOrder) : name(name), properties(properties), propertyInsertionOrder(propertyInsertionOrder){};
    //         llvm::Value *codegen();

    //         std::string getName();
    //         std::map<std::string, std::string> getProperties();
    //         std::vector<std::string> getPropertyInsertionOrder();
    //     };

    //     class StructValueExpression : public Expression
    //     {
    //     private:
    //         std::map<std::string, unique_ptr<Expression>> properties;
    //         std::vector<std::string> propertyInsertionOrder;

    //     public:
    //         StructValueExpression(std::map<std::string, unique_ptr<Expression>> properties, std::vector<std::string> propertyInsertionOrder) : properties(std::move(properties)), propertyInsertionOrder(propertyInsertionOrder){};
    //         llvm::Value *codegen();
    //         std::map<std::string, unique_ptr<Expression>> getProperties();
    //         std::vector<std::string> getPropertyInsertionOrder();
    //     };

    //     class IfStatement : public Expression
    //     {
    //     private:
    //         std::vector<unique_ptr<Expression>> conditions;
    //         std::vector<ssc::TokenType> conditionSeparators;
    //         unique_ptr<CodeBlock> then;

    //     public:
    //         IfStatement(std::vector<unique_ptr<Expression>> conditions, std::vector<ssc::TokenType> conditionSeparators, unique_ptr<CodeBlock> then) : conditions(std::move(conditions)), conditionSeparators(conditionSeparators), then(std::move(then)){};
    //         llvm::Value *codegen();

    //         std::vector<unique_ptr<Expression>> getConditions();
    //         std::vector<ssc::TokenType> getConditionSeparators();
    //         CodeBlock *getThen();
    //     };

    //     class ReturnStatement : public Expression
    //     {
    //     private:
    //         unique_ptr<Expression> value;

    //     public:
    //         ReturnStatement(unique_ptr<Expression> value) : value(std::move(value)){};
    //         llvm::Value *codegen();

    //         Expression *getValue();
    //     };

    //     class FunctionCallExpression : public Expression
    //     {
    //     private:
    //         std::string name;
    //         std::vector<unique_ptr<Expression>> params;

    //     public:
    //         FunctionCallExpression(std::string name, std::vector<unique_ptr<Expression>> params) : name(name), params(std::move(params)){};
    //         llvm::Value *codegen();

    //         std::string getName();
    //         const std::vector<unique_ptr<Expression>> &getParams();
    //     };

    //     class ImportStatement : public Expression
    //     {
    //     private:
    //         std::string path;

    //     public:
    //         ImportStatement(std::string path) : path(path){};
    //         llvm::Value *codegen();
    //         std::string getPath();
    //     };

    //     typedef std::vector<unique_ptr<Node>> Nodes;

    // } // namespace ssc

#endif