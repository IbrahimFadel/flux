#ifndef CODE_GENERATION_H
#define CODE_GENERATION_H

#include "parser.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Verifier.h>
#include <llvm/Support/raw_ostream.h>

#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/LegacyPassManagers.h>
#include <llvm/Support/TargetRegistry.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm/Target/TargetOptions.h>
#include <llvm/Target/TargetMachine.h>
#include <llvm/Bitcode/BitcodeWriter.h>
#include <llvm/Bitcode/BitcodeReader.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>

#include <llvm/Transforms/InstCombine/InstCombine.h>
#include <llvm/Transforms/Scalar.h>
#include <llvm/Transforms/Scalar/GVN.h>

enum Scopes
{
    global,
    function
};

static llvm::LLVMContext context;
static std::unique_ptr<llvm::Module> module = std::make_unique<llvm::Module>("Module", context);
static llvm::IRBuilder<> builder(context);
static std::unique_ptr<llvm::legacy::FunctionPassManager> function_pass_manager;

static std::map<std::string, llvm::Value *> global_variables;
static std::map<std::string, std::map<std::string, llvm::Value *>> function_variables;
static std::string current_function;

static std::map<std::string, Function_Node *> functions;
static Scopes scope = Scopes::global;

void module_to_bin();
void code_gen(std::vector<std::unique_ptr<Node>> nodes);
void code_gen_node(std::unique_ptr<Node> node);
void initialize_fpm();
static llvm::Type *ss_type_to_llvm_type(Variable_Types type);
static llvm::AllocaInst *create_entry_block_alloca(llvm::Function *TheFunction,
                                                   const std::string &VarName);
static llvm::Type *bitwidth_to_llvm_type(unsigned int bitwidth);

static llvm::Value *error_v(const char *Str);

#endif