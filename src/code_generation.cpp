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
    module_to_bin();

    auto writer = new llvm::AssemblyAnnotationWriter();
    // module->print(*os, writer);
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
    case Node_Types::TypeCastNode:
    {
        auto to = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        to->code_gen();
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
    case Variable_Types::type_i64:
        return llvm::ConstantInt::get(context, llvm::APInt(64, (int)value, true));
    case Variable_Types::type_i32:
        return llvm::ConstantInt::get(context, llvm::APInt(32, (int)value, true));
    case Variable_Types::type_i16:
        return llvm::ConstantInt::get(context, llvm::APInt(16, (int)value, true));
    case Variable_Types::type_i8:
        return llvm::ConstantInt::get(context, llvm::APInt(8, (int)value, true));
    case Variable_Types::type_float:
        return llvm::ConstantFP::get(context, llvm::APFloat((float)value));
    case Variable_Types::type_double:
        return llvm::ConstantFP::get(context, llvm::APFloat((double)value));
    case Variable_Types::type_bool:
        return llvm::ConstantInt::get(context, llvm::APInt(1, (int)value, true));
    default:
        return llvm::ConstantFP::get(context, llvm::APFloat(value));
    }
    return llvm::ConstantInt::get(context, llvm::APInt(32, (int)value, true));
}

llvm::Value *Binary_Expression_Node::code_gen()
{
    llvm::Value *l = lhs->code_gen();
    llvm::Value *r = rhs->code_gen();
    if (l == 0 || r == 0)
        return 0;

    auto pre_loaded_l_type = l->getType();
    auto pre_loaded_r_type = r->getType();

    llvm::Value *loaded_l;
    llvm::Value *loaded_r;

    if (pre_loaded_l_type->isPointerTy())
    {
        loaded_l = builder.CreateLoad(l);
    }
    else
    {
        loaded_l = l;
    }

    if (pre_loaded_r_type->isPointerTy())
    {
        loaded_r = builder.CreateLoad(r);
    }
    else
    {
        loaded_r = r;
    }

    auto l_type = loaded_l->getType();
    auto r_type = loaded_r->getType();

    if (l_type->isDoubleTy() || r_type->isDoubleTy() || l_type->isFloatTy() || r_type->isFloatTy())
    {
        if (op == "+")
            return builder.CreateFAdd(loaded_l, loaded_r, "addtmp");
        if (op == "-")
            return builder.CreateFSub(loaded_l, loaded_r, "subtmp");
        if (op == "*")
            return builder.CreateFMul(loaded_l, loaded_r, "multmp");
    }
    else
    {
        // // auto l_ptr = builder.CreateAlloca(llvm::Type::getInt32Ty(context), NULL, "x");
        // auto new_l = llvm::ConstantInt::get(context, llvm::APInt(32, (int)12, true));
        // // builder.CreateStore(new_l, l_ptr);
        // auto new_r = llvm::ConstantInt::get(context, llvm::APInt(8, (int)5, true));

        // builder.CreateAdd(new_l, new_r, "test");
        // if (l_type->isIntegerTy())
        // {
        unsigned int l_bitwidth = l_type->getIntegerBitWidth();
        unsigned int r_bitwidth = r_type->getIntegerBitWidth();

        llvm::Value *new_l;
        llvm::Value *new_r;

        if (l_bitwidth < r_bitwidth)
        {
            auto new_type = bitwidth_to_llvm_type(r_bitwidth);
            new_l = builder.CreateBitCast(loaded_l, new_type);
            new_r = r;
        }
        else if (r_bitwidth < l_bitwidth)
        {
            auto new_type = bitwidth_to_llvm_type(l_bitwidth);
            new_r = builder.CreateBitCast(loaded_r, new_type);
            new_l = l;
        }

        // }
        // else
        // {
        if (op == "+")
            return builder.CreateAdd(loaded_l, loaded_r, "addtmp");
        if (op == "-")
            return builder.CreateSub(loaded_l, loaded_r, "subtmp");
        if (op == "*")
            return builder.CreateMul(loaded_l, loaded_r, "multmp");
        // }
    }
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

    auto arg_types = functions[callee]->get_arg_types();

    std::vector<llvm::Value *> args_v;
    for (unsigned int i = 0, e = args.size(); i != e; i++)
    {
        auto v = args[i]->code_gen();
        auto loaded = builder.CreateLoad(v);
        args_v.push_back(loaded);
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
    if (proto->get_name() == "main")
    {
        construct_global_variable_assign_function();
    }

    llvm::Function *the_function = proto->code_gen();
    if (the_function == 0)
        return 0;

    llvm::BasicBlock *bb = llvm::BasicBlock::Create(context, "entry", the_function);
    builder.SetInsertPoint(bb);

    functions[current_function] = this;

    scope = Scopes::function;

    proto->create_argument_allocas(the_function);

    if (proto->get_name() == "main")
    {
        std::vector<llvm::Value *> call_args(0);
        auto callee_f = module->getFunction(global_variable_assign_function_name);
        builder.CreateCall(callee_f, call_args);
    }

    for (auto &node : body)
    {
        code_gen_node(std::move(node));
    }

    if (llvm::verifyFunction(*the_function, &llvm::outs()))
    {
        llvm::raw_ostream &os = llvm::outs();
        os << '\n'
           << '\n';
        llvm::StringRef o_name = "out.ll";
        std::error_code ec;
        llvm::raw_fd_ostream *out_stream = new llvm::raw_fd_ostream(o_name, ec);
        auto writer = new llvm::AssemblyAnnotationWriter();
        module->print(os, writer);
        module->print(*out_stream, writer);
        exit(-1);
    }

    function_pass_manager->run(*the_function);

    return the_function;
}

llvm::Value *Return_Node::code_gen()
{
    auto ptr = value->code_gen();
    auto val = builder.CreateLoad(ptr);
    return builder.CreateRet(val);
}

llvm::Value *Variable_Node::code_gen()
{
    if (scope == Scopes::global)
    {
        auto llvm_type = ss_type_to_llvm_type(type);
        auto constant_temp = get_zeroed_variable(llvm_type);
        auto var = new llvm::GlobalVariable(*module.get(), llvm_type, false, llvm::GlobalValue::CommonLinkage, constant_temp, name);

        global_variables[name] = var;
        global_variables_awaiting_initialization[name] = std::move(value);
        return var;
    }
    else if (scope == Scopes::function)
    {
        llvm::Value *alloc = new llvm::AllocaInst(ss_type_to_llvm_type(type), NULL, name, builder.GetInsertBlock());
        auto v = value->code_gen();
        llvm::Value *loaded_v;
        if (v->getType()->isPointerTy())
        {
            loaded_v = builder.CreateLoad(v);
            builder.CreateStore(loaded_v, alloc);
        }
        else
        {
            builder.CreateStore(v, alloc);
        }

        functions[current_function]->set_variables(name, alloc);
        return alloc;
    }
}

llvm::Value *Variable_Expression_Node::code_gen()
{
    auto ptr = functions[current_function]->get_variable(name);
    if (ptr == 0)
    {
        auto glob_ptr = global_variables[name];
        if (glob_ptr == 0)
        {
            char err_msg[150];
            sprintf(err_msg, "Reference to undefined variable: %s", name.c_str());
            error_v(err_msg);
            exit(-1);
        }
        return glob_ptr;
    }
    return ptr;
}

llvm::Value *Type_Cast_Node::code_gen()
{
    auto llvm_type = variable_type_to_llvm_ptr_type(new_type);
    auto v = value->code_gen();
    auto newv = builder.CreateBitCast(v, llvm_type);
    functions[current_function]->set_variables(v->getName().str(), newv);
    return newv;
}

llvm::Value *construct_global_variable_assign_function()
{
    auto ret = llvm::Type::getVoidTy(context);

    std::vector<llvm::Type *> arg_types(0);

    llvm::FunctionType *function_type = llvm::FunctionType::get(ret, false);
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, global_variable_assign_function_name, module.get());

    if (f->getName() != global_variable_assign_function_name)
    {
        f->eraseFromParent();
        f = module->getFunction(global_variable_assign_function_name);
        cout << "reassigning" << endl;

        if (!f->empty())
        {
            error_v("'__assign_global_variables' is a reserved function name");
            exit(-1);
        }
    }

    current_function = global_variable_assign_function_name;

    if (f == 0)
        return 0;

    auto the_function = module->getFunction(global_variable_assign_function_name);

    auto bb = llvm::BasicBlock::Create(context, "entry", the_function);
    builder.SetInsertPoint(bb);

    scope = Scopes::function;

    // ! Go through 'global_variables_awaiting_initialization' and initialize

    auto it = global_variables_awaiting_initialization.begin();
    while (it != global_variables_awaiting_initialization.end())
    {
        auto v = it->second->code_gen();
        builder.CreateStore(v, global_variables[it->first]);

        it++;
    }

    builder.CreateRetVoid();

    if (llvm::verifyFunction(*the_function, &llvm::outs()))
    {
        llvm::raw_ostream &os = llvm::outs();
        os << '\n'
           << '\n';
        llvm::StringRef o_name = "out.ll";
        std::error_code ec;
        llvm::raw_fd_ostream *out_stream = new llvm::raw_fd_ostream(o_name, ec);
        auto writer = new llvm::AssemblyAnnotationWriter();
        module->print(os, writer);
        module->print(*out_stream, writer);
        exit(-1);
    }

    function_pass_manager->run(*the_function);

    return the_function;
}

llvm::Value *get_ptr_or_value_with_type(llvm::Value *val, Variable_Types type)
{
    //! all sandscript types are not pointers right now, so just load them
    //! in the future when variables can be pointers, implement logic:
    //! If llvm val is pointer and function wants pointer return pointer, else load and return
    // auto llvm_type = val->getType();
    // if (llvm_type->isPointerTy())
    // {
    //     iff(type == Variable_Types::)
    // }
    // return builder.CreateLoad(val);
    return nullptr;
}

llvm::Type *variable_type_to_llvm_ptr_type(Variable_Types type)
{
    switch (type)
    {
    case Variable_Types::type_i64:
        return llvm::Type::getInt64PtrTy(context);
    case Variable_Types::type_i32:
        return llvm::Type::getInt32PtrTy(context);
    case Variable_Types::type_i16:
        return llvm::Type::getInt16PtrTy(context);
    case Variable_Types::type_i8:
        return llvm::Type::getInt8PtrTy(context);
    default:
        break;
    }
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
    case Variable_Types::type_i64:
        return llvm::Type::getInt64Ty(context);
    case Variable_Types::type_i32:
        return llvm::Type::getInt32Ty(context);
    case Variable_Types::type_i16:
        return llvm::Type::getInt16Ty(context);
    case Variable_Types::type_i8:
        return llvm::Type::getInt8Ty(context);
    case Variable_Types::type_float:
        return llvm::Type::getFloatTy(context);
    case Variable_Types::type_double:
        return llvm::Type::getDoubleTy(context);
    case Variable_Types::type_bool:
        return llvm::Type::getInt1Ty(context);
    default:
        break;
    }
}

llvm::Type *bitwidth_to_llvm_type(unsigned int bitwidth)
{
    switch (bitwidth)
    {
    case 64:
        return llvm::Type::getInt64Ty(context);
    case 32:
        return llvm::Type::getInt32Ty(context);
    case 16:
        return llvm::Type::getInt16Ty(context);
    case 8:
        return llvm::Type::getInt8Ty(context);
    default:
        return llvm::Type::getInt32Ty(context);
    }
}

llvm::Constant *get_zeroed_variable(llvm::Type *type)
{
    if (type->isIntegerTy())
    {
        auto bitwidth = type->getIntegerBitWidth();
        return llvm::ConstantInt::get(context, llvm::APInt(bitwidth, (int)0, true));
    }
    else if (type->isDoubleTy())
    {
        return llvm::ConstantFP::get(context, llvm::APFloat((double)0));
    }
    else if (type->isFloatTy())
    {
        return llvm::ConstantFP::get(context, llvm::APFloat((float)0));
    }
    else
    {
        return llvm::ConstantInt::get(context, llvm::APInt(32, (int)0, true));
    }
}

llvm::Value *error_v(const char *str)
{
    cout << "LogError: " << str << endl;
    return 0;
}