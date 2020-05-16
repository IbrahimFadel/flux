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

static llvm::LLVMContext context;
static llvm::Module *TheModule = new llvm::Module("example", context);
static llvm::IRBuilder<> Builder(context);
static std::map<std::string, llvm::Value *> NamedValues;

void generate_llvm_ir(std::vector<Node *> nodes)
{
  LLVMInitializeNativeTarget();

  llvm::raw_ostream *os = &llvm::outs();
  llvm::StringRef oname = "llvm_ir";
  std::error_code EC;
  llvm::raw_fd_ostream *out = new llvm::raw_fd_ostream(oname, EC);

  for (auto &node : nodes)
  {
    llvm::Value *v;
    llvm::Function *prototype;
    llvm::Function *function_body;

    switch (node->type)
    {
    case Node_Types::ConstantDeclarationNode:
    {
      v = std::get<Constant_Declaration_Node *>(node->constant_declaration_node)->code_gen();
      v->print(*os);
      cout << endl;
      v->print(*out);
      out->write('\n');

      break;
    }
    case Node_Types::FunctionDeclarationNode:
    {
      prototype = std::get<Function_Declaration_Node *>(node->function_declaration_node)->code_gen_prototype();
      function_body = std::get<Function_Declaration_Node *>(node->function_declaration_node)->code_gen_function_body(prototype);
      prototype->print(*os);
      cout << endl;
      prototype->print(*out);
      out->write('\n');
      break;
    }
    default:
      break;
    }
  }
}

Number evaluate_number_expression(Number_Expression_Node expr)
{
  Number num;
  num.type = Number_Types::FloatNumber;

  std::vector<float> term_values;
  std::vector<float> term_values_values;
  float value = 0;
  for (auto &term : expr.terms)
  {
    float term_value = 0;
    int i = 0;
    if (term.ops.size() > 0)
    {
      for (auto &op : term.ops)
      {
        if (op == "*")
        {
          if (term_values.size() > 0)
          {
            term_value = term_values[term_values.size() - 1] * std::stof(term.numbers[i + 1]);
          }
          else
          {
            term_value = std::stof(term.numbers[i]) * std::stof(term.numbers[i + 1]);
          }

          term_values.push_back(term_value);
        }
        else if (op == "/")
        {
          if (term_values.size() > 0)
          {
            term_value = term_values[term_values.size() - 1] / std::stof(term.numbers[i + 1]);
          }
          else
          {
            term_value = std::stof(term.numbers[i]) / std::stof(term.numbers[i + 1]);
          }

          term_values.push_back(term_value);
        }
        i++;
      }
      term_values_values.push_back(term_values[term_values.size() - 1]);
      term_values.clear();
    }
    else
    {
      term_value = std::stof(term.numbers[0]);
      term_values_values.push_back(term_value);
    }
  }

  for (auto &val : term_values_values)
  {
    value += val;
  }

  num.float_number = value;

  return num;
}

llvm::Value *Return_Node::code_gen()
{
  if (return_expression->type == Expression_Types::NumberExpression)
  {
  }
}

llvm::Value *Expression_Node::code_gen()
{
  if (type == Expression_Types::NumberExpression)
  {
    Number num = evaluate_number_expression(std::get<Number_Expression_Node>(number_expression));
    switch (num.type)
    {
    case Number_Types::FloatNumber:
      return llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
    default:
      break;
    }
  }
  else if (type == Expression_Types::StringExpression)
  {
  }
}

llvm::Function *Function_Declaration_Node::code_gen_prototype()
{
  std::vector<llvm::Type *> Doubles(parameters.size(), llvm::Type::getDoubleTy(context));

  llvm::FunctionType *FT = llvm::FunctionType::get(llvm::Type::getDoubleTy(context), Doubles, false);

  llvm::Function *F = llvm::Function::Create(FT, llvm::Function::ExternalLinkage, name, TheModule);

  unsigned Idx = 0;
  std::map<std::string, Variable_Types>::iterator it;
  std::vector<std::string> names;
  for (it = parameters.begin(); it != parameters.end(); it++)
  {
    names.push_back(it->first);
  }
  for (auto &Arg : F->args())
  {
    Arg.setName(names[Idx]);
  }

  return F;
}

llvm::Function *Function_Declaration_Node::code_gen_function_body(llvm::Function *proto)
{
  llvm::Function *TheFunction = TheModule->getFunction(proto->getName());

  if (!TheFunction)
    TheFunction = code_gen_prototype();
  if (!TheFunction)
    return nullptr;
  if (!TheFunction->empty())
  {
    std::cerr << "Error" << std::endl;
    return nullptr;
  }

  llvm::BasicBlock *BB = llvm::BasicBlock::Create(context, "entry", TheFunction);
  Builder.SetInsertPoint(BB);

  NamedValues.clear();
  for (auto &Arg : TheFunction->args())
  {
    NamedValues[Arg.getName()] = &Arg;
  }

  for (auto &node : then.nodes)
  {
    switch (node->type)
    {
    case Node_Types::ReturnNode:
    {
      if (std::get<Return_Node *>(node->return_node)->return_expression->type == Expression_Types::NumberExpression)
      {
        Return_Node *return_node = std::get<Return_Node *>(node->return_node);
        Expression_Node *expr = return_node->return_expression.get();

        llvm::Value *v = expr->code_gen();
        Builder.CreateRet(v);
        llvm::verifyFunction(*TheFunction);

        return TheFunction;
      }
      break;
    }
    default:
      break;
    }
  }

  TheFunction->eraseFromParent();

  return nullptr;
}

llvm::Value *Constant_Declaration_Node::code_gen()
{
  switch (type)
  {
  case Variable_Types::FloatType:
  {
    Number num = evaluate_number_expression(std::get<Number_Expression_Node>(expression->number_expression));
    return llvm::ConstantFP::get(context, llvm::APFloat(std::get<float>(num.float_number)));
    break;
  }
  default:
    return llvm::ConstantFP::get(context, llvm::APFloat(2.5));
    break;
  }
}

float evaluate_float_expression(std::unique_ptr<Expression_Node> expression)
{
  return 1.0;
}