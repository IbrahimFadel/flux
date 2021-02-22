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
        unique_ptr<llvm::Module> mod;
        llvm::IRBuilder<> builder;
        std::string currentFunctionName;
        std::map<std::string, FunctionDeclaration *> functions;
        std::string currentlyPreferredType;

    public:
        CodegenContext() : builder(ctx){};
        void init(std::string moduleName);
        void error(std::string msg);

        template <typename T>
        void print(T v)
        {
            v->print(llvm::outs());
            llvm::outs() << '\n';
        }

        llvm::Type *ssTypeToLLVMType(std::string type);
        llvm::Type *ssBaseTypeToLLVMType(std::string type);
        bool isTypeSigned(std::string type);

        void setCurrentlyPreferredType(std::string ty) { currentlyPreferredType = ty; }
        std::string getCurrentlyPreferredType() { return currentlyPreferredType; }
        void setCurrentFunctionName(std::string name) { currentFunctionName = name; }
        std::string getCurrentFunctionName() { return currentFunctionName; }
        void setFunction(std::string name, FunctionDeclaration *fn) { functions[name] = fn; }
        FunctionDeclaration *getFunction(std::string name) { return functions[name]; }
        const unique_ptr<llvm::Module> &getModule() { return mod; }
        llvm::IRBuilder<> *getBuilder() { return &builder; }
        llvm::LLVMContext *getCtx() { return &ctx; }
    };
} // namespace ssc

#endif