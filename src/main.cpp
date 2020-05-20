#include <iostream>
#include <fstream>
#include <istream>

#include "lexer.h"
#include "parser.h"
#include "code_generation.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm-c/Target.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/IR/Verifier.h>
#include <llvm/IR/PassManager.h>
#include <llvm/Bitcode/BitcodeWriter.h>
#include <llvm/Bitcode/BitcodeReader.h>
#include <llvm/IR/GlobalValue.h>

using namespace llvm;

using std::cout;
using std::endl;

std::string get_file_content(const char *path)
{
  std::ifstream in(path);
  std::string contents((std::istreambuf_iterator<char>(in)),
                       std::istreambuf_iterator<char>());
  return contents;
}

static llvm::LLVMContext context;
static llvm::Module *TheModule = new llvm::Module("Module", context);
static llvm::IRBuilder<> Builder(context);
static std::map<std::string, llvm::Value *> NamedValues;

void run()
{
  // LLVMInitializeNativeTarget();
  // raw_ostream *os = &llvm::outs();
  // Value *v = new llvm::GlobalVariable(Type::getFloatTy(context), true, GlobalValue::CommonLinkage, ConstantFP::get(context, APFloat(2.5)), "x");
  // v->print(*os, false);

  // Value *loc = new llvm::Varia
}

int main()
{
  // run();

  // llvm::raw_ostream *os = &llvm::outs();
  // llvm::StringRef oname = "llvm_ir";
  // std::error_code EC;
  // llvm::raw_fd_ostream *out = new llvm::raw_fd_ostream(oname, EC);

  // llvm::StringRef source_file_name("test.ss");
  // TheModule->setSourceFileName(source_file_name);

  // Value *v = ConstantFP::get(context, APFloat(3.5));

  // std::cout << "ss >>> ";
  // std::string in;
  // std::getline(std::cin, in);

  // auto tokens = get_tokens(in);
  // auto nodes = parse_tokens(tokens);
  // generate_llvm_ir(nodes);
  std::string file_content = get_file_content("test.ss");

  auto tokens = get_tokens(file_content);
  print_tokens(tokens);

  std::vector<Node *> nodes = parse_tokens(tokens);
  print_nodes(nodes);
  generate_llvm_ir(nodes);

  return 0;
}