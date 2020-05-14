#include "code_generation.h"

#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Module.h>
#include <llvm/IR/Value.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm-c/Target.h>
#include <llvm/Support/raw_ostream.h>
#include <llvm/Support/FileSystem.h>

using namespace llvm;

static LLVMContext context;
static Module *TheModule = new Module("example", context);
static IRBuilder<> Builder(context);
static std::map<std::string, Value *> NamedValues;

void generate_llvm_ir(std::vector<Node *> nodes)
{
  LLVMInitializeNativeTarget();

  raw_ostream *os = &outs();
  StringRef oname = "llvm_ir";
  std::error_code EC;
  raw_fd_ostream *out = new raw_fd_ostream(oname, EC);

  for (auto &node : nodes)
  {
    Value *v;

    switch (node->type)
    {
    case Node_Types::ConstantDeclarationNode:
    {
      v = std::get<Constant_Declaration_Node *>(node->constant_declaration_node)->code_gen();
      break;
    }
    default:
      break;
    }
    Value *sum = Builder.CreateAdd(v, ConstantFP::get(context, APFloat(3.8)), "addtmp");
    sum->print(*os, false);
    sum->print(*out, false);
  }
}

llvm::Value *Constant_Declaration_Node::code_gen()
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {
    float expression_value = evaluate_float_expression(std::move(expression));
    return ConstantFP::get(context, APFloat(expression_value));
    break;
  }
  default:
    return ConstantFP::get(context, APFloat(2.5));
    break;
  }
}

float evaluate_float_expression(std::unique_ptr<Expression_Node> expression)
{
  return 1.0;
}