#ifndef CODE_GENERATION_H
#define CODE_GENERATION_H

#include "options.h"
#include "common.h"
#include "parser.h"
#include "dependency_tree.h"

#include <llvm/IR/Module.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/IR/AssemblyAnnotationWriter.h>

#include <llvm/Bitcode/BitcodeWriter.h>

#include <llvm/Transforms/InstCombine/InstCombine.h>
#include <llvm/Transforms/Scalar.h>
#include <llvm/Transforms/Scalar/GVN.h>
#include <llvm/Transforms/Utils.h>

#include <llvm/Support/TargetSelect.h>
#include <llvm/Support/TargetRegistry.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/Support/Host.h>
#include <llvm/Support/raw_ostream.h>

#include <llvm/Target/TargetOptions.h>
#include <llvm/Target/TargetMachine.h>

#include <lld/Common/Driver.h>

static CompilerOptions compiler_options;
static llvm::LLVMContext context;
// static llvm::Module *module;
static llvm::IRBuilder<> builder(context);
static unique_ptr<llvm::legacy::FunctionPassManager> fpm;

static std::map<std::string, Function_Declaration *> functions;
static std::string current_function_name;
static llvm::Type *currently_preferred_type = llvm::Type::getInt32Ty(context);

static bool print_function_declared = false;
static std::map<fs::path, llvm::Module *> files_with_modules_already_generated;

static std::map<std::string, llvm::StructType *> llvm_struct_types;
static std::map<std::string, std::map<std::string, std::string>> struct_properties;

// unique_ptr<llvm::Module> code_gen_nodes(const Nodes &nodes, CompilerOptions options, unique_ptr<Program> parent_program);
void create_module(const Nodes &nodes, CompilerOptions options, std::string path, Dependency_Tree *tree, llvm::Module *mod);
void module_to_obj(llvm::Module *mod, std::string output_path);
static void declare_imported_functions(Dependency_Tree *tree, fs::path path, llvm::Module *mod);
static void code_gen_node(const unique_ptr<Node> &node, llvm::Module *mod);
static void initialize_fpm(llvm::Module *mod);
static void create_function_param_allocas(llvm::Function *f, std::map<std::string, std::string> params);
static llvm::Value *create_entry_block_alloca(llvm::Function *function, const std::string &name, llvm::Type *type);
static llvm::Type *ss_type_to_llvm_type(std::string type);
static llvm::Type *ss_base_type_to_llvm_type(std::string type);
static void declare_function(std::string name, std::vector<llvm::Type *> param_types, llvm::Type *return_type, llvm::Module *mod);
static void print_t(llvm::Type *ty);
static void print_v(llvm::Value *v);
void print_module(llvm::Module *mod);
void write_module_to_file(llvm::Module *mod, std::string path);
static void fatal_error(std::string msg);

static llvm::Function *code_gen_function_prototype(std::map<std::string, std::string> params, std::string return_type, std::string function_name, llvm::Module *mod);
static llvm::Value *code_gen_struct_variable_declaration(std::string name, std::string type, Struct_Value_Expression *value, llvm::Module *mod);

// unique_ptr<Module> code_gen_nodes(const Nodes &nodes, CompilerOptions compiler_options);

// #include "options.h"
// #include "ast.h"
// #include "parser.h"

// #include <llvm/IR/Module.h>
// #include <llvm/IR/IRBuilder.h>
// #include <llvm/IR/Value.h>
// #include <llvm/IR/Verifier.h>
// #include <llvm/IR/AssemblyAnnotationWriter.h>
// #include <llvm/IR/LegacyPassManager.h>
// #include <llvm/IR/LegacyPassManagers.h>
// #include <llvm/IR/LegacyPassNameParser.h>

// #include <llvm/Transforms/InstCombine/InstCombine.h>
// #include <llvm/Transforms/Scalar.h>
// #include <llvm/Transforms/Scalar/GVN.h>
// #include <llvm/Transforms/Utils.h>

// #include <llvm/Support/Host.h>
// #include <llvm/Support/raw_ostream.h>
// #include <llvm/Support/TargetSelect.h>
// #include <llvm/Support/TargetRegistry.h>

// #include <llvm/ExecutionEngine/ExecutionEngine.h>

// #include <llvm/Pass.h>

// using namespace llvm;

// enum Scope
// {
//     global,
//     function,
// };

// static LLVMContext context;
// static std::map<int, unique_ptr<Module>> modules;
// static int current_module_pointer = 0;
// static llvm::IRBuilder<> builder(context);
// static unique_ptr<llvm::legacy::FunctionPassManager> fpm;
// static CompilerOptions compiler_options;

// static std::map<std::string, Function_Node *> functions;
// static std::string current_function_name;

// static Scope scope;
// static bool wants_reference = false;

// static std::map<std::string, unique_ptr<Expression_Node>> global_variables_awaiting_initialization;
// static std::string global_variable_assign_function_name = "__assign_global_variables";

// static Variable_Type currently_preferred_type = Variable_Type::type_i32;
// static std::map<std::string, Type *> object_types;
// static std::map<std::string, std::map<std::string, Variable_Type>> object_type_properties;

// unique_ptr<Module> code_gen_nodes(const Nodes &nodes, CompilerOptions compiler_options);
// void module_to_obj(unique_ptr<llvm::Module> m);
// static Value *code_gen_node(const unique_ptr<Node> &node);
// static void initialize_function_pass_manager();
// static void declare_printf();
// static Value *load_if_ptr(Value *v);
// static void define_object_properties(Variable_Declaration_Node *var, Value *ptr, unique_ptr<Expression_Node> expr);
// static Value *code_gen_primitive_variable_declaration(Variable_Declaration_Node *var);
// static Value *code_gen_array_variable_declaration(Variable_Declaration_Node *var);
// static Value *code_gen_string_variable_declaration(Variable_Declaration_Node *var);
// static Value *initialize_string(String_Expression *);
// static void initialize_string_type();
// static void print_v(Value *v);
// static void print_t(Value *v);
// static void print_current_module();

// static bool is_reference_type(Variable_Type type);
// static Value *create_entry_block_alloca(Function *function, const std::string &name, Type *type);
// static Type *variable_type_to_llvm_type(Variable_Type type, std::string object_type_name = "");
// static Variable_Type llvm_type_to_variable_type(llvm::Type *type);
// static void error(const char *arg);

#endif