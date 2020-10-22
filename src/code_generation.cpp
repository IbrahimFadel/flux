#include "code_generation.h"

#include <lld/Common/Driver.h>

void module_to_bin()
{
    auto TargetTriple = llvm::sys::getDefaultTargetTriple();
    llvm::InitializeAllTargetInfos();
    llvm::InitializeAllTargets();
    llvm::InitializeAllTargetMCs();
    llvm::InitializeAllAsmParsers();
    llvm::InitializeAllAsmPrinters();

    std::string Error;
    auto Target = llvm::TargetRegistry::lookupTarget(TargetTriple, Error);

    if (!Target)
    {
        llvm::errs() << Error;
        return;
    }

    auto CPU = "generic";
    auto Features = "";

    llvm::TargetOptions opt;
    auto RM = llvm::Optional<llvm::Reloc::Model>();
    llvm::TargetMachine *TargetMachine = Target->createTargetMachine(TargetTriple, CPU, Features, opt, RM);

    module->setDataLayout(TargetMachine->createDataLayout());
    module->setTargetTriple(TargetTriple);

    auto Filename = "out";
    std::error_code EC;
    llvm::raw_fd_ostream dest(Filename, EC, llvm::sys::fs::OF_None);

    if (EC)
    {
        llvm::errs() << "Could not open file: " << EC.message();
        return;
    }

    llvm::legacy::PassManager pass;
    auto ASMFILE = llvm::CGFT_AssemblyFile;
    auto OBJFILE = llvm::CGFT_ObjectFile;
    // auto bin = llvm::CGFT

    if (TargetMachine->addPassesToEmitFile(pass, dest, nullptr, OBJFILE))
    {
        llvm::errs() << "TargetMachine can't emit a file of this type";
    }

    pass.run(*module);
    dest.flush();

    // const char *emu_argv[] = {
    //     "ld.lld",
    //     "objfile.o",
    //     "-o",
    //     "exec",
    // };
    // int argc = sizeof(emu_argv) / sizeof(emu_argv[0]);
    // const char **argv = (const char **)emu_argv;
    // std::vector<const char *> args(argv, argv + argc);
    // lld::elf::link(args, false, llvm::outs(), llvm::outs());
    // lld::elf::link(args, false, llvm::outs(), llvm::outs());
}

void code_gen(std::vector<std::unique_ptr<Node>> nodes)
{
    llvm::raw_ostream *os = &llvm::outs();
    llvm::StringRef o_name = "out.ll";
    std::error_code ec;
    llvm::raw_fd_ostream *out_stream = new llvm::raw_fd_ostream(o_name, ec);

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node));
    }
    module_to_bin();

    auto writer = new llvm::AssemblyAnnotationWriter();
    module->print(*os, writer);
    module->print(*out_stream, writer);
}

void code_gen_node(std::unique_ptr<Node> node)
{
    switch (node->type)
    {
    case Node_Types::FunctionDeclarationNode:
    {
        auto function = std::get<std::unique_ptr<Function_Node>>(std::move(node->function_node));
        auto v = function->code_gen();
        break;
    }
    case Node_Types::VariableDeclarationNode:
    {
        auto expr = std::get<std::unique_ptr<Variable_Node>>(std::move(node->variable_node));
        auto v = expr->code_gen();
        break;
    }
    case Node_Types::ReturnNode:
    {
        auto ret = std::get<std::unique_ptr<Return_Node>>(std::move(node->return_node));
        ret->code_gen();
        break;
    }
    case Node_Types::CallExpressionNode:
    {
        auto call = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        call->code_gen();
        break;
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

    // l->print(llvm::outs());
    // r->print(llvm::outs());

    // if (lhs->type != Variable_Types::type_int && rhs->type != Variable_Types::type_int)
    // {
    //     if (op == "+")
    //         return builder.CreateFAdd(l, r, "addtmp");
    //     if (op == "-")
    //         return builder.CreateFSub(l, r, "addtmp");
    //     if (op == "*")
    //         return builder.CreateFMul(l, r, "addtmp");
    // }
    // else
    // {
    if (op == "+")
        return builder.CreateAdd(l, r, "addtmp");
    if (op == "-")
        return builder.CreateSub(l, r, "subtmp");
    if (op == "*")
        return builder.CreateMul(l, r, "multmp");
    // }
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
        auto v = args[i]->code_gen();
        args_v.push_back(v);
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
    }

    current_function = name;

    return f;
}

llvm::Function *Function_Node::code_gen()
{
    llvm::Function *the_function = proto->code_gen();
    if (the_function == 0)
        return 0;

    llvm::BasicBlock *bb = llvm::BasicBlock::Create(context, "entry", the_function);
    builder.SetInsertPoint(bb);

    functions[current_function] = this;

    proto->create_argument_allocas(the_function);

    for (auto &node : body)
    {
        code_gen_node(std::move(node));
    }

    llvm::verifyFunction(*the_function);

    return the_function;
}

llvm::Value *Return_Node::code_gen()
{
    builder.CreateRet(value->code_gen());
    return 0;
}

llvm::Value *Variable_Node::code_gen()
{
    llvm::Value *alloc = new llvm::AllocaInst(ss_type_to_llvm_type(type), NULL, name, builder.GetInsertBlock());
    auto v = value->code_gen();
    auto store = new llvm::StoreInst(v, alloc, builder.GetInsertBlock());
    functions[current_function]->set_variables(name, alloc);
    return alloc;
}

llvm::Value *Variable_Expression_Node::code_gen()
{
    auto ptr = functions[current_function]->get_variable(name);
    return builder.CreateLoad(ptr, ptr->getName().str());
}

llvm::AllocaInst *create_entry_block_alloca(llvm::Function *TheFunction,
                                            const std::string &VarName)
{
    llvm::IRBuilder<> TmpB(&TheFunction->getEntryBlock(),
                           TheFunction->getEntryBlock().begin());
    return TmpB.CreateAlloca(llvm::Type::getInt32Ty(context), 0,
                             VarName.c_str());
}

void Prototype_Node::create_argument_allocas(llvm::Function *f)
{
    llvm::Function::arg_iterator AI = f->arg_begin();
    for (unsigned idx = 0, e = arg_names.size(); idx != e; ++idx, ++AI)
    {
        auto alloc = create_entry_block_alloca(f, arg_names[idx]);
        builder.CreateStore(AI, alloc);
        functions[f->getName().str()]->set_variables(arg_names[idx], alloc);
    }
}

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