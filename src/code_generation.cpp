#include "code_generation.h"

void code_gen(std::vector<std::unique_ptr<Node>> nodes)
{
    for (auto &node : nodes)
    {
        code_gen_node(std::move(node));
    }
}

void code_gen_node(std::unique_ptr<Node> node, bool function_body, llvm::BasicBlock *bb)
{
    llvm::raw_ostream *os = &llvm::outs();

    llvm::StringRef o_name = "llvm_ir";
    std::error_code ec;
    llvm::raw_fd_ostream *out_stream = new llvm::raw_fd_ostream(o_name, ec);

    switch (node->type)
    {
    case Node_Types::FunctionDeclarationNode:
    {
        auto function = std::get<std::unique_ptr<Function_Node>>(std::move(node->function_node));
        auto v = function->code_gen();
        v->print(*os);
        v->print(*out_stream);
        break;
    }
    case Node_Types::VariableDeclarationNode:
    {
        auto expr = std::get<std::unique_ptr<Variable_Node>>(std::move(node->variable_node));
        if (function_body)
        {
            auto v = expr->code_gen(bb);
        }
    }
    default:
        break;
    }
}

llvm::Value *Number_Expression_Node::code_gen()
{
    switch (variable_type)
    {
    case Variable_Types::type_int:
        return llvm::ConstantInt::get(context, llvm::APInt(32, (int)value, true));
        break;

    default:
        return llvm::ConstantFP::get(context, llvm::APFloat(value));
        break;
    }
    return llvm::ConstantInt::get(context, llvm::APInt(32, (int)value, true));
}

llvm::Value *Binary_Expression_Node::code_gen()
{
    llvm::Value *l = lhs->code_gen();
    llvm::Value *r = rhs->code_gen();
    if (l == 0 || r == 0)
        return 0;

    if (lhs->type != Variable_Types::type_int && rhs->type != Variable_Types::type_int)
    {
        if (op == "+")
            return builder.CreateFAdd(l, r, "addtmp");
        if (op == "-")
            return builder.CreateFSub(l, r, "addtmp");
        if (op == "*")
            return builder.CreateFMul(l, r, "addtmp");
    }
    else
    {
        if (op == "+")
            return builder.CreateAdd(l, r, "addtmp");
        if (op == "-")
            return builder.CreateSub(l, r, "addtmp");
        if (op == "*")
            return builder.CreateMul(l, r, "addtmp");
    }
}

llvm::Value *Call_Expression_Node::code_gen()
{
    llvm::Function *callee_f = module->getFunction(callee);
    if (callee_f == 0)
        return error_v("Unknown function referenced");

    if (callee_f->arg_size() != args.size())
        return error_v("Incorrect number of arguments passed");

    std::vector<llvm::Value *> args_v;
    for (unsigned int i = 0, e = args.size(); i != e; i++)
    {
        args_v.push_back(args[i]->code_gen());
        if (args_v.back() == 0)
            return 0;
    }

    return builder.CreateCall(callee_f, args_v, "calltmp");
}

llvm::Function *Prototype_Node::code_gen()
{
    std::vector<llvm::Type *> param_types;
    for (auto &param_type : arg_types)
    {
        auto llvm_type = ss_type_to_llvm_type(param_type);
        param_types.push_back(llvm_type);
    }

    auto ret = ss_type_to_llvm_type(return_type);

    llvm::FunctionType *function_type = llvm::FunctionType::get(ret, param_types, false);
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, name, module.get());

    if (f->getName() != name)
    {
        f->eraseFromParent();
        f = module->getFunction(name);

        if (!f->empty())
        {
            error_v("Redefinition of function");
            return 0;
        }

        if (f->arg_size() != arg_names.size())
        {
            error_v("Redefinition of function with different number of arguments");
            return 0;
        }
    }

    unsigned idx = 0;
    for (llvm::Function::arg_iterator ai = f->arg_begin(); idx != arg_names.size(); ai++, idx++)
    {
        ai->setName(arg_names[idx]);
        // ! remember NamedValues map
    }

    return f;
}

llvm::Function *Function_Node::code_gen()
{
    llvm::Function *the_function = proto->code_gen();
    if (the_function == 0)
        return 0;

    llvm::BasicBlock *bb = llvm::BasicBlock::Create(context, "entry", the_function);
    builder.SetInsertPoint(bb);

    for (auto &node : body)
    {
        code_gen_node(std::move(node), true, bb);
    }

    // if(llvm::Value *return_value = body)
    builder.CreateRetVoid();

    llvm::verifyFunction(*the_function);

    return the_function;
}

llvm::Value *Variable_Node::code_gen(llvm::BasicBlock *bb)
{
    llvm::Value *alloc = new llvm::AllocaInst(ss_type_to_llvm_type(type), NULL, name, bb);
    auto v = value->code_gen();
    auto store = new llvm::StoreInst(v, alloc, bb);
    // auto load = new llvm::LoadInst(alloca, name, bb);

    return alloc;
}

llvm::Value *Variable_Expression_Node::code_gen() {}

llvm::Type *ss_type_to_llvm_type(Variable_Types type)
{
    switch (type)
    {
    case Variable_Types::type_int:
        return llvm::Type::getInt32Ty(context);
    default:
        break;
    }
}

llvm::Value *error_v(const char *str)
{
    cout << "LogError: " << str << endl;
    return 0;
}