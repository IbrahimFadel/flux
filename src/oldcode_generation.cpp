#include "code_generation.h"

#include <lld/Common/Driver.h>

void module_to_obj(std::shared_ptr<llvm::Module> module, std::string path)
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

    auto filename = path;
    std::error_code ec;
    llvm::raw_fd_ostream dest(filename, ec, llvm::sys::fs::F_None);

    if (ec)
    {
        llvm::errs() << "Could not open file: " << ec.message() << '\n';
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
    function_pass_manager = std::make_unique<llvm::legacy::FunctionPassManager>(modules[current_module].get());
    function_pass_manager->add(llvm::createInstructionCombiningPass());
    function_pass_manager->add(llvm::createReassociatePass());
    function_pass_manager->add(llvm::createGVNPass());
    function_pass_manager->add(llvm::createCFGSimplificationPass());
    function_pass_manager->doInitialization();
}

std::shared_ptr<llvm::Module> code_gen_nodes(std::vector<std::unique_ptr<Node>> nodes)
{
    char module_name[20];
    sprintf(module_name, "Module%d", current_module);
    auto module = std::make_shared<llvm::Module>(module_name, context);
    modules.push_back(std::move(module));

    initialize_fpm();

    define_putchar();

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node));
    }

    if (current_module == 0)
    {
        char o_name[100];
        sprintf(o_name, "%s/obj%d.o", build_dir.c_str(), current_module);
        module_to_obj(modules[current_module], o_name);

        auto writer = new llvm::AssemblyAnnotationWriter();
        modules[current_module]->print(llvm::outs(), writer);
    }

    // // module->print(*os, writer);
    // modules[current_module]->print(*out_stream, writer);

    return modules[current_module];
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
    case Node_Types::ImportNode:
    {
        auto import = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        import->code_gen();
        break;
    }
    case Node_Types::VariableDeclarationNode:
    {
        auto expr = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
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
    case Node_Types::AssignmentNode:
    {
        auto assignment = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        assignment->code_gen();
        break;
    }
    case Node_Types::IfNode:
    {
        auto if_node = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        if_node->code_gen();
        break;
    }
    case Node_Types::ForNode:
    {
        auto for_node = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        for_node->code_gen();
        break;
    }
    case Node_Types::ObjectNode:
    {
        auto object_node = std::get<std::unique_ptr<Expression_Node>>(std::move(node->expression_node));
        object_node->code_gen();
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
        if (op == "<")
            return builder.CreateFCmpOLT(loaded_l, loaded_r, "lttmp");
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
        if (op == "<")
            return builder.CreateICmpSLT(loaded_l, loaded_r, "lttmp");
        // }
    }
    return 0;
}

llvm::Value *Call_Expression_Node::code_gen()
{
    llvm::Function *callee_f = modules[current_module]->getFunction(callee);
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

    auto arg_types_it = callee_f->arg_begin();

    std::vector<llvm::Value *> args_v;
    for (unsigned int i = 0, e = args.size(); i != e; i++)
    {
        auto ty = arg_types_it->getType();
        auto v = args[i]->code_gen();
        llvm::Value *loaded;
        if (v->getType()->isPointerTy())
        {
            loaded = builder.CreateLoad(v);
        }
        else
        {
            if (ty->isIntegerTy())
            {
                auto arg_bitwidth = ty->getIntegerBitWidth();
                auto given_bitwidth = v->getType()->getIntegerBitWidth();
                if (arg_bitwidth != given_bitwidth)
                {
                    auto casted = builder.CreateIntCast(v, bitwidth_to_llvm_type(arg_bitwidth), true);
                    loaded = casted;
                }
                else
                {
                    loaded = v;
                }
            }
            else
            {
                loaded = v;
            }
        }

        args_v.push_back(loaded);

        arg_types_it++;
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
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, name, modules[current_module].get());

    if (f->getName() != name)
    {
        f->eraseFromParent();
        f = modules[current_module]->getFunction(name);

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
    if (proto->get_name() == "main" && global_variables_awaiting_initialization.size() > 0)
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

    if (proto->get_name() == "main" && global_variables_awaiting_initialization.size() > 0)
    {
        std::vector<llvm::Value *> call_args(0);
        auto callee_f = modules[current_module]->getFunction(global_variable_assign_function_name);
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
        auto writer = new llvm::AssemblyAnnotationWriter();
        modules[current_module]->print(os, writer);
        exit(-1);
    }

    // function_pass_manager->run(*the_function);

    return the_function;
}

llvm::Value *Return_Node::code_gen()
{
    auto ptr = value->code_gen();
    llvm::Value *loaded;
    if (ptr->getType()->isPointerTy())
    {
        loaded = builder.CreateLoad(ptr, ptr->getName().str());
    }
    else
    {
        loaded = ptr;
    }

    return builder.CreateRet(loaded);
    // builder.CreateStore(loaded, functions[current_function]->get_return_value_ptr());
    // return builder.CreateBr(functions[current_function]->get_end_bb());
    // return builder.CreateBr(functions[current_function]->get_end_bb());
}

llvm::Value *Variable_Node::code_gen()
{
    if (scope == Scopes::global)
    {
        auto llvm_type = ss_type_to_llvm_type(type);
        auto constant_temp = get_zeroed_variable(llvm_type);
        auto var = new llvm::GlobalVariable(*modules[current_module].get(), llvm_type, false, llvm::GlobalValue::CommonLinkage, constant_temp, name);

        global_variables[name] = var;
        global_variables_awaiting_initialization[name] = std::move(value);
        return var;
    }
    else if (scope == Scopes::function)
    {
        if (type == Variable_Types::type_string)
        {
            auto str = value->code_gen();
            functions[current_function]->set_variables(name, str);
            return str;
        }
        llvm::Type *ty;
        if (type == Variable_Types::type_object)
        {
            ty = get_object_type(object_type);
        }
        else
        {
            ty = ss_type_to_llvm_type(type);
        }

        llvm::Value *alloc = new llvm::AllocaInst(ty, 0, name, builder.GetInsertBlock());
        if (type == Variable_Types::type_object)
        {
            define_object_properties(alloc, std::move(value));
            return alloc;
        }

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

    return 0;
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
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, global_variable_assign_function_name, modules[current_module].get());

    if (f->getName() != global_variable_assign_function_name)
    {
        f->eraseFromParent();
        f = modules[current_module]->getFunction(global_variable_assign_function_name);

        if (!f->empty())
        {
            error_v("'__assign_global_variables' is a reserved function name");
            exit(-1);
        }
    }

    current_function = global_variable_assign_function_name;

    if (f == 0)
        return 0;

    auto the_function = modules[current_module]->getFunction(global_variable_assign_function_name);

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
        auto writer = new llvm::AssemblyAnnotationWriter();
        modules[current_module]->print(os, writer);
        exit(-1);
    }

    function_pass_manager->run(*the_function);

    return the_function;
}

llvm::Value *Assignment_Node::code_gen()
{
    return builder.CreateStore(value->code_gen(), functions[current_function]->get_variable(name));
}

llvm::Value *Condition_Expression::code_gen()
{
    auto l = lhs->code_gen();
    auto r = rhs->code_gen();
    llvm::Value *l_loaded = l;
    llvm::Value *r_loaded = r;
    if (l->getType()->isPointerTy())
        l_loaded = builder.CreateLoad(l);
    if (r->getType()->isPointerTy())
        r_loaded = builder.CreateLoad(r);

    switch (op)
    {
    case Token_Types::tok_compare_eq:
    {
        return builder.CreateICmpEQ(l_loaded, r_loaded, "ifcond");
    }
    default:
        return error_v("Could not determine the operator in if statement condition");
    }
}

llvm::Value *If_Node::code_gen()
{
    //TODO A function should have a 'return expression' to be kept track of and evaluated at the end. All return nodes will just update that expression's value
    //TODO At the end of the function, manually generate a return instruction with the evaluated return expression.
    /**
     * if(x == y) {
     *  return x;
     * }
     * 
     * TRANSLATES TO
     * 
     * %retval = alloca i32 ; get the type from function's return type
     * %ifcond = icmp eq i32* %x, %y
     * br i1 %ifcond, label %then, label %else
     * 
     * then:
     *  %x_loaded = load i32, i32* %x
     *  store i32 %x_loaded, i32* %retval
     *  br %continue
     * else:
     *  br %continue
     * continue:
     *  %retval_loaded = load i32, i32* retval
     *  ret i32 %retval_loaded
     * 
     */

    std::vector<llvm::Value *> cmps;
    for (auto &condition : conditions)
    {
        auto cmp = condition->code_gen();
        cmps.push_back(cmp);
    }

    llvm::Value *last_comparison = cmps[0]; //? If there end up not being any condition_seperators, this will be there instead
    int i = 0;
    for (auto &sep : condition_seperators)
    {
        switch (sep)
        {
        case Token_Types::tok_and:
        {
            if (i == 0)
                last_comparison = builder.CreateAnd(cmps[i], cmps[i + 1]);
            else
                last_comparison = builder.CreateAnd(last_comparison, cmps[i + 1]);
            break;
        }
        case Token_Types::tok_or:
        {
            if (i == 0)
                last_comparison = builder.CreateOr(cmps[i], cmps[i + 1]);
            else
                last_comparison = builder.CreateOr(last_comparison, cmps[i + 1]);
        }
        default:
            break;
        }
        i++;
    }

    auto true_val = llvm::ConstantInt::get(llvm::Type::getInt1Ty(context), (uint64_t)1, false);
    auto cmp = builder.CreateICmpEQ(last_comparison, true_val);

    auto the_function = builder.GetInsertBlock()->getParent();

    llvm::BasicBlock *then_bb = llvm::BasicBlock::Create(context, "then", the_function);
    llvm::BasicBlock *else_bb = llvm::BasicBlock::Create(context, "else");
    llvm::BasicBlock *merge_bb = llvm::BasicBlock::Create(context, "continue");

    auto cond_br = builder.CreateCondBr(cmp, then_bb, else_bb);

    builder.SetInsertPoint(then_bb);

    for (auto &node : then)
    {
        code_gen_node(std::move(node));
    }

    builder.CreateBr(merge_bb);

    then_bb = builder.GetInsertBlock();

    the_function->getBasicBlockList().push_back(else_bb);
    builder.SetInsertPoint(else_bb);

    // builder.CreateAlloca(ss_type_to_llvm_type(functions[current_function]->get_proto()->get_return_type()), NULL, "hi");

    builder.CreateBr(merge_bb);

    else_bb = builder.GetInsertBlock();

    the_function->getBasicBlockList().push_back(merge_bb);
    builder.SetInsertPoint(merge_bb);

    return 0;
}

llvm::Value *Import_Node::code_gen()
{
    char file_path[500];
    sprintf(file_path, "%s/%s", project_root.c_str(), path.c_str());
    // auto file_content = get_file_content(file_path);
    auto file_content = get_file_content(file_path);
    auto tokens = get_tokens(file_content);
    auto nodes = parse_tokens(tokens);

    current_module++;

    auto module = code_gen_nodes(std::move(nodes));
    char o_name[100];
    sprintf(o_name, "%s/obj%d.o", build_dir.c_str(), current_module);
    module_to_obj(module, o_name);

    current_module -= 1;

    //? Iterate over the imported module's functions and make a forward definition in the main module
    auto cur = module->getFunctionList().begin();
    auto end = module->getFunctionList().end();
    while (cur != end)
    {
        auto f_name = cur->getName().str();
        auto f_type = cur->getFunctionType();
        auto f = llvm::Function::Create(f_type, llvm::Function::ExternalLinkage, f_name, modules[current_module].get());
        cur++;
    }

    return 0;
}

llvm::Value *String_Expression::code_gen()
{
    if (value == "\\n")
        return builder.CreateGlobalStringPtr("\n");
    if (value == "\\t")
        return builder.CreateGlobalStringPtr("\t");
    return builder.CreateGlobalStringPtr(value);
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
        error_v("Could not convert variable type to llvm ptr type");
        exit(1);
        break;
    }
}

llvm::AllocaInst *create_entry_block_alloca(llvm::Function *TheFunction,
                                            const std::string &VarName, llvm::Type *type)
{
    llvm::IRBuilder<> TmpB(&TheFunction->getEntryBlock(),
                           TheFunction->getEntryBlock().begin());
    return TmpB.CreateAlloca(type, 0,
                             VarName.c_str());
}

void Prototype_Node::create_argument_allocas(llvm::Function *f)
{
    llvm::Function::arg_iterator AI = f->arg_begin();
    for (unsigned idx = 0, e = arg_names.size(); idx != e; ++idx, ++AI)
    {
        auto alloc = create_entry_block_alloca(f, arg_names[idx], ss_type_to_llvm_type(arg_types[idx]));
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
    case Variable_Types::type_string:
        return llvm::Type::getInt8Ty(context);
    case Variable_Types::type_object:
        // return objects[typ]
        return llvm::Type::getInt64Ty(context);
    default:
        error_v("Could not convert variable type to llvm type");
        exit(1);
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

void define_putchar()
{
    std::vector<llvm::Type *> param_types;
    param_types.push_back(llvm::Type::getInt8Ty(context));
    llvm::FunctionType *f_type = llvm::FunctionType::get(llvm::Type::getInt32Ty(context), param_types, false);
    auto f = llvm::Function::Create(f_type, llvm::Function::ExternalLinkage, "putchar", modules[current_module].get());
}

llvm::StringRef Variable_Node::code_gen_str()
{
    // return builder.CreateGlobalString("hello", name);
    // value->
    return "hello";
}

llvm::Value *For_Node::code_gen()
{
    auto the_function = builder.GetInsertBlock()->getParent();
    auto var = variable->code_gen();
    auto cond = condition->code_gen();
    auto loop_bb = llvm::BasicBlock::Create(context, "loop", the_function);
    auto continue_bb = llvm::BasicBlock::Create(context, "continue", the_function);
    auto br = builder.CreateCondBr(cond, loop_bb, continue_bb);
    builder.SetInsertPoint(loop_bb);
    action->code_gen();
    for (auto &expr : body)
    {
        code_gen_node(std::move(expr));
    }

    auto new_cond = condition->code_gen();
    auto repeat = builder.CreateCondBr(new_cond, loop_bb, continue_bb);

    builder.SetInsertPoint(continue_bb);

    return 0;
}

llvm::Value *Object_Node::code_gen()
{
    auto it = properties.begin();

    std::vector<llvm::Type *> members;
    while (it != properties.end())
    {
        members.push_back(ss_type_to_llvm_type(it->second));
        it++;
    }

    llvm::ArrayRef<llvm::Type *> struct_properties(members);
    auto struct_type = llvm::StructType::create(context, struct_properties, name, false);

    objects[name] = struct_type;

    return 0;
}

llvm::Value *Object_Expression_Node::code_gen()
{
    // builder.CreateGEP()
    // auto it = properties.begin();
    // cout << "obj expr" << endl;
    // cout << type << endl;

    return 0;
}

void define_object_properties(llvm::Value *ptr, std::unique_ptr<Expression_Node> expr)
{
    cout << "Store values in object" << endl;

    auto properties = expr->get_properties();
    auto it = properties.begin();
    while (it != properties.end())
    {
        cout << it->first << endl;
        it++;
    }

    std::vector<llvm::Value *> values;
    llvm::APInt zero(32, 0);
    llvm::APInt one(32, 1);
    // This is the array offset into RaviGCObject*
    values.push_back(
        llvm::Constant::getIntegerValue(llvm::Type::getInt32Ty(context), zero));
    // This is the field offset
    values.push_back(
        llvm::Constant::getIntegerValue(llvm::Type::getInt32Ty(context), zero));

    auto prop = builder.CreateGEP(ptr, values, "ptr_to_val");

    builder.CreateStore(llvm::ConstantInt::get(llvm::Type::getInt16Ty(context), llvm::APInt(16, 10)), prop);
}

llvm::Type *get_object_type(std::string object_type_name)
{
    return objects[object_type_name];
}

llvm::Value *error_v(const char *str)
{
    cout << "LogError: " << str << endl;
    return 0;
}

void print(llvm::Value *v)
{
    v->print(llvm::outs());
    llvm::outs() << '\n';
}

void print_ty(llvm::Type *ty)
{
    ty->print(llvm::outs());
    llvm::outs() << '\n';
}