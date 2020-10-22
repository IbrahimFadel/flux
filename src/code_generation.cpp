#include "code_generation.h"

#include <lld/Common/Driver.h>

void module_to_bin()
{
    auto target_triple = llvm::sys::getDefaultTargetTriple();

    llvm::InitializeAllTargetInfos();
    llvm::InitializeAllTargets();
    llvm::InitializeAllTargetMCs();
    llvm::InitializeAllAsmParsers();
    llvm::InitializeAllAsmPrinters();

    std::string error;
    auto target = llvm::TargetRegistry::lookupTarget(target_triple, error);
    if (!target)
    {
        llvm::errs() << error;
        return;
    }

    auto cpu = "generic";
    auto features = "";

    llvm::TargetOptions opt;
    auto rm = llvm::Optional<llvm::Reloc::Model>();
    auto target_machine = target->createTargetMachine(target_triple, cpu, features, opt, rm);

    module->setDataLayout(target_machine->createDataLayout());
    module->setTargetTriple(target_triple);

    auto filename = "out.o";
    std::error_code ec;
    llvm::raw_fd_ostream dest(filename, ec, llvm::sys::fs::F_None);

    if (ec)
    {
        llvm::errs() << "Could not open file: " << ec.message();
        return;
    }

    llvm::legacy::PassManager pass;
    auto file_type = llvm::CGFT_ObjectFile;

    if (target_machine->addPassesToEmitFile(pass, dest, nullptr, file_type))
    {
        llvm::errs() << "TargetMachine can't emit a file of this type";
        return;
    }

    pass.run(*module);
    dest.flush();
}

void initialize_fpm()
{
    function_pass_manager = std::make_unique<llvm::legacy::FunctionPassManager>(module.get());
    function_pass_manager->add(llvm::createInstructionCombiningPass());
    function_pass_manager->add(llvm::createReassociatePass());
    function_pass_manager->add(llvm::createGVNPass());
    function_pass_manager->add(llvm::createCFGSimplificationPass());
    function_pass_manager->doInitialization();
}

void code_gen(std::vector<std::unique_ptr<Node>> nodes)
{
    llvm::raw_ostream *os = &llvm::outs();
    llvm::StringRef o_name = "out.ll";
    std::error_code ec;
    llvm::raw_fd_ostream *out_stream = new llvm::raw_fd_ostream(o_name, ec);

    initialize_fpm();

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node));
    }
    // module_to_bin();

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
    {
        error_v("Unknown function referenced");
        exit(-1);
    }

    if (callee_f->arg_size() != args.size())
    {
        error_v("Incorrect number of arguments passed");
        exit(-1);
    }

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
            exit(-1);
        }

        if (f->arg_size() != arg_names.size())
        {
            error_v("Redefinition of function with different number of arguments");
            exit(-1);
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

    auto fn = std::make_unique<Function_Node>(std::move(proto), std::move(body));
    auto scope_node = std::make_unique<Node>();
    scope_node->type = Node_Types::VariableDeclarationNode;
    scope_node->function_node = std::move(fn);
    scope = std::move(scope_node);

    auto fn_ref = std::get<std::unique_ptr<Function_Node>>(std::move(scope->function_node));
    llvm::Function *the_function = fn_ref->proto->code_gen();
    if (the_function == 0)
        return 0;

    llvm::BasicBlock *bb = llvm::BasicBlock::Create(context, "entry", the_function);
    builder.SetInsertPoint(bb);

    functions[current_function] = this;

    fn_ref->proto->create_argument_allocas(the_function);

    for (auto &node : fn_ref->body)
    {
        code_gen_node(std::move(node));
    }

    llvm::verifyFunction(*the_function);

    // function_pass_manager->run(*the_function);

    return the_function;
}

llvm::Value *Return_Node::code_gen()
{
    builder.CreateRet(value->code_gen());
    return 0;
}

llvm::Value *Variable_Node::code_gen()
{
    if (!scope)
    {
        auto var = new llvm::GlobalVariable(*module.get(), llvm::Type::getInt32Ty(context), false, llvm::GlobalValue::CommonLinkage, 0, name);
        global_variables[name] = var;
        return var;
    }
    else
    {
        llvm::Value *alloc = new llvm::AllocaInst(ss_type_to_llvm_type(type), NULL, name, builder.GetInsertBlock());
        auto v = value->code_gen();
        auto store = new llvm::StoreInst(v, alloc, builder.GetInsertBlock());
        functions[current_function]->set_variables(name, alloc);
        return alloc;
    }
}

llvm::Value *Variable_Expression_Node::code_gen()
{
    auto ptr = functions[current_function]->get_variable(name);
    if (ptr == 0)
    {
        ptr = global_variables[name];
        if (ptr == 0)
        {
            char err_msg[150];
            sprintf(err_msg, "Reference to undefined variable: %s", name.c_str());
            error_v(err_msg);
            exit(-1);
        }
    }
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