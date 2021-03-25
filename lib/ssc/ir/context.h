#ifndef SSC_IR_CONTEXT_H
#define SSC_IR_CONTEXT_H

#include <memory>
#include <string>
#include <map>

namespace ssc
{
    class CodegenContext;
}

#include "ast/nodes.h"
#include "driver/options.h"

#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/Value.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/AssemblyAnnotationWriter.h"
#include "llvm/IR/Verifier.h"

// #include <llvm/Bitcode/BitcodeWriter.h>

// #include <llvm/Transforms/InstCombine/InstCombine.h>
// #include <llvm/Transforms/Scalar.h>
// #include <llvm/Transforms/Scalar/GVN.h>
// #include <llvm/Transforms/Utils.h>

// #include <llvm/Support/TargetSelect.h>
// #include <llvm/Support/TargetRegistry.h>
// #include <llvm/Support/FileSystem.h>
// #include <llvm/Support/Host.h>
// #include <llvm/Support/raw_ostream.h>

// #include <llvm/Target/TargetOptions.h>
// #include <llvm/Target/TargetMachine.h>

using std::unique_ptr;

namespace ssc
{
    class CodegenContext
    {
    private:
        llvm::LLVMContext ctx;
        llvm::Module *mod;
        llvm::IRBuilder<> builder;
        std::string currentFunctionName;
        std::map<std::string, ASTFunctionDeclaration *> functions;
        std::string currentlyPreferredType;
        unique_ptr<Options> &compilerOptions;

    public:
        //  mod = new llvm::Module(moduleName, ctx);

        // auto _builder = llvm::IRBuilder(ctx);

        CodegenContext(std::string moduleName, unique_ptr<Options> &compilerOptions) : builder(ctx), compilerOptions(compilerOptions)
        {
            mod = new llvm::Module(moduleName, ctx);
        };
        void error(std::string msg);
        void warning(std::string msg);
        void printModule()
        {
            auto writer = new llvm::AssemblyAnnotationWriter();
            mod->print(llvm::outs(), writer);
        }

        template <typename T>
        void print(T v)
        {
            v->print(llvm::outs());
            llvm::outs() << '\n';
        }

        llvm::Type *ssTypeToLLVMType(std::string type);
        llvm::Type *ssBaseTypeToLLVMType(std::string type);
        bool isTypeSigned(std::string type);
        llvm::Value *implicityTypecastExpression(llvm::Value *v, std::string currentType, llvm::Type *targetType);

        void setCurrentlyPreferredType(std::string ty) { currentlyPreferredType = ty; }
        std::string getCurrentlyPreferredType() { return currentlyPreferredType; }
        void setCurrentFunctionName(std::string name) { currentFunctionName = name; }
        std::string getCurrentFunctionName() { return currentFunctionName; }
        void setFunction(std::string name, ASTFunctionDeclaration *fn) { functions[name] = fn; }
        ASTFunctionDeclaration *getFunction(std::string name) { return functions[name]; }
        llvm::Module *getModule() { return mod; }
        llvm::IRBuilder<> *getBuilder() { return &builder; }
        llvm::LLVMContext *getCtx() { return &ctx; }
        unique_ptr<Options> &getCompilerOptions() { return compilerOptions; }
    };
} // namespace ssc

#endif