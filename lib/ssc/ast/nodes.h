#ifndef SSC_AST_NODES_H
#define SSC_AST_NODES_H

#include <vector>
#include <string>
#include <memory>
#include <map>
#include <variant>

namespace ssc
{
    class ASTExpression;
    class ASTFunctionDeclaration;
    class ASTVariableDeclaration;
    struct Parameter;
    class ASTReturnStatement;
} // namespace ssc

#include "ir/context.h"
#include "llvm/IR/Value.h"

#include "lexer.h"

using std::unique_ptr;

namespace ssc
{
    class ASTNode
    {
    private:
    public:
        virtual llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext) = 0;
    };

    class ASTExpression : public ASTNode
    {
    private:
    public:
        virtual llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext) = 0;
        virtual std::string getType() = 0;
    };

    class ASTNumberExpression : public ASTExpression
    {
    private:
        double value;
        std::string type;

    public:
        ASTNumberExpression(double value, std::string type) : value(value), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        double getValue() { return value; }
        std::string getType() { return type; }
    };

    class ASTBinaryOperationExpression : public ASTExpression
    {
    private:
        unique_ptr<ASTExpression> lhs;
        unique_ptr<ASTExpression> rhs;
        TokenType op;
        std::string type;

        llvm::Value *codegenBinopSumDiffProdQuot(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op);
        llvm::Value *codegenBinopEq(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs);
        llvm::Value *codegenBinopComp(const unique_ptr<CodegenContext> &codegenContext, unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op);

    public:
        ASTBinaryOperationExpression(unique_ptr<ASTExpression> lhs, unique_ptr<ASTExpression> rhs, TokenType op, std::string type) : lhs(std::move(lhs)), rhs(std::move(rhs)), op(op), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        unique_ptr<ASTExpression> &getLHS() { return lhs; }
        unique_ptr<ASTExpression> &getRHS() { return rhs; }
        std::string getType() { return type; }
    };

    class ASTVariableReferenceExpression : public ASTExpression
    {
    private:
        std::string name;
        std::string type;
        bool isMut;

    public:
        ASTVariableReferenceExpression(std::string name, std::string type, bool isMut) : name(name), type(type), isMut(isMut){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getName() { return name; }
        std::string getType() { return type; }
        bool getIsMut() { return isMut; }
    };

    struct Parameter
    {
        bool mut;
        std::string type;
        std::string name;
    };

    class ASTFunctionDeclaration : public ASTNode
    {
    private:
        bool pub;
        std::string name;
        std::vector<Parameter> parameters;
        std::string returnType;
        std::vector<unique_ptr<ASTNode>> then;

        std::map<std::string, ASTVariableDeclaration *> variables;
        std::map<std::string, llvm::Value *> mutables;
        std::map<std::string, llvm::Value *> constants;

        llvm::Function *codegenPrototype(const unique_ptr<CodegenContext> &codegenContext);
        void createFunctionParamAllocas(const unique_ptr<CodegenContext> &codegenContext, llvm::Function *f);

    public:
        ASTFunctionDeclaration(bool pub, std::string name, std::vector<Parameter> parameters, std::string returnType, std::vector<unique_ptr<ASTNode>> then) : pub(pub), name(name), parameters(parameters), returnType(returnType), then(std::move(then)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        void setVariable(std::string name, ASTVariableDeclaration *v) { variables[name] = v; }
        ASTVariableDeclaration *getVariable(std::string name) { return variables[name]; }
        std::map<std::string, llvm::Value *> getConstants() { return constants; };
        std::map<std::string, llvm::Value *> getMutables() { return mutables; };
        void setMutable(std::string name, llvm::Value *val);
        llvm::Value *getMutable(std::string name);
        void setConstant(std::string name, llvm::Value *val);
        llvm::Value *getConstant(std::string name);
        bool getPub();
        std::string getName();
        std::vector<Parameter> getParameters();
        std::string getReturnType();
        const std::vector<unique_ptr<ASTNode>> &getThen();
    };

    class ASTVariableDeclaration : public ASTNode
    {
    private:
        bool pub;
        bool mut;
        std::string type;
        std::string name;
        unique_ptr<ASTExpression> value;

    public:
        ASTVariableDeclaration(bool pub, bool mut, std::string type, std::string name, unique_ptr<ASTExpression> value) : pub(pub), mut(mut), type(type), name(name), value(std::move(value)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getType() { return type; };
        std::string getName() { return name; };
        const unique_ptr<ASTExpression> &getValue() { return value; };
    };

    class ASTReturnStatement : public ASTNode
    {
    private:
        unique_ptr<ASTExpression> value;

    public:
        ASTReturnStatement(unique_ptr<ASTExpression> value) : value(std::move(value)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        const unique_ptr<ASTExpression> &getValue() { return value; }
    };

    class ASTTypecastExpression : public ASTExpression
    {
    private:
        unique_ptr<ASTExpression> value;
        std::string type;

    public:
        ASTTypecastExpression(unique_ptr<ASTExpression> value, std::string type) : value(std::move(value)), type(type){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);

        std::string getType() { return type; }
    };

    class ASTIfStatement : public ASTNode
    {
    private:
        unique_ptr<ASTExpression> condition;
        std::vector<unique_ptr<ASTNode>> then;

    public:
        ASTIfStatement(unique_ptr<ASTExpression> condition, std::vector<unique_ptr<ASTNode>> then) : condition(std::move(condition)), then(std::move(then)){};
        llvm::Value *codegen(const unique_ptr<CodegenContext> &codegenContext);
    };

    typedef std::vector<unique_ptr<ASTNode>> Nodes;

}; // namespace ssc

#endif