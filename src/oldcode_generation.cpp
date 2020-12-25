#include "code_generation.h"

unique_ptr<Module> code_gen_nodes(const Nodes &nodes, CompilerOptions opts)
{
    compiler_options = opts;
    modules[current_module_pointer] = std::make_unique<Module>("TheModule", context);

    if (compiler_options.optimize)
        initialize_function_pass_manager();

    declare_printf();
    // initialize_string_type();

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node));
    }

    print_current_module();

    return std::move(modules[current_module_pointer]);
}

void initialize_string_type()
{
    std::map<std::string, Variable_Type> properties;
    properties["length"] = Variable_Type::type_i32;
    properties["value"] = Variable_Type::type_array;

    auto it = properties.begin();

    std::vector<llvm::Type *> members;
    while (it != properties.end())
    {
        auto ty = variable_type_to_llvm_type(it->second);
        members.push_back(ty);

        object_type_properties["string"][it->first] = it->second;

        it++;
    }

    llvm::ArrayRef<llvm::Type *> struct_properties(members);
    auto struct_type = llvm::StructType::create(context, struct_properties, "string", false);

    object_types["string"] = struct_type;

    struct_type->print(llvm::outs());
    llvm::outs() << '\n';
}

void declare_printf()
{
    std::vector<llvm::Type *> param_types;
    param_types.push_back(llvm::Type::getInt8PtrTy(context));
    llvm::FunctionType *f_type = llvm::FunctionType::get(llvm::Type::getInt32Ty(context), param_types, true);
    auto f = llvm::Function::Create(f_type, llvm::Function::ExternalLinkage, "printf", modules[current_module_pointer].get());
}

void module_to_obj(unique_ptr<llvm::Module> m)
{
    cout << "compiling module to obj: " << compiler_options.output_path << endl;
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
        exit(1);
    }

    auto CPU = "generic";
    auto features = "";

    llvm::TargetOptions opt;
    auto rm = llvm::Optional<llvm::Reloc::Model>();
    auto target_machine = target->createTargetMachine(target_triple, CPU, features, opt, rm);

    m->setDataLayout(target_machine->createDataLayout());
    m->setTargetTriple(target_triple);

    std::error_code ec;
    llvm::raw_fd_ostream dest(compiler_options.output_path, ec, llvm::sys::fs::F_None);
    if (ec)
    {
        llvm::errs() << "Could not open file: " << ec.message();
        exit(1);
    }

    llvm::legacy::PassManager pass;
    auto file_type = llvm::CGFT_ObjectFile;

    if (target_machine->addPassesToEmitFile(pass, dest, nullptr, file_type))
    {
        errs() << "Target Machine cannot emit a file of this type";
        exit(1);
    }

    pass.run(*m.get());
    dest.flush();

    // const char *emu_argv[] = {
    //     "ld.lld",
    //     compiler_options.output_path.c_str(),
    //     "-o",
    //     "../testytest",
    // };
    // int argc = sizeof(emu_argv) / sizeof(emu_argv[0]);
    // const char **argv = (const char **)emu_argv;
    // std::vector<const char *> args(argv, argv + argc);
    // lld::elf::link(args, false, outs(), outs());
}

Value *code_gen_node(const unique_ptr<Node> &node)
{
    node->code_gen();
    return 0;
}

void initialize_function_pass_manager()
{
    fpm = std::make_unique<llvm::legacy::FunctionPassManager>(modules[current_module_pointer].get());
    fpm->add(llvm::createInstructionCombiningPass());
    fpm->add(llvm::createReassociatePass());
    // fpm->add(llvm::createDeadCodeEliminationPass());
    // fpm->add(llvm::createGVNPass());
    fpm->add(llvm::createCFGSimplificationPass());
    fpm->add(llvm::createPromoteMemoryToRegisterPass());

    fpm->doInitialization();
}

void print_v(Value *v)
{
    v->print(outs());
    outs() << '\n';
}

void print_t(Type *t)
{
    t->print(outs());
    outs() << '\n';
}

Value *Function_Node::code_gen()
{
    auto function = prototype->code_gen_proto();
    if (function == 0)
        return 0;

    auto entry = BasicBlock::Create(context, "entry", function);
    builder.SetInsertPoint(entry);

    current_function_name = prototype->get_name();
    functions[current_function_name] = this;

    scope = Scope::function;

    prototype->create_argument_allocas(function);

    if (prototype->get_name() == "main" && global_variables_awaiting_initialization.size() > 0)
    {
        std::vector<llvm::Value *> call_args(0);
        auto callee_f = modules[current_module_pointer]->getFunction(global_variable_assign_function_name);
        builder.CreateCall(callee_f, call_args);
    }

    currently_preferred_type = prototype->get_return_type();

    then->code_gen();

    // const llvm::Function::BasicBlockListType &tst = function->getBasicBlockList();
    // auto bb_it = tst.begin();
    // auto bb_end = tst.end();
    // while (bb_it != bb_end)
    // {
    //     cout << "BB" << endl;
    //     const llvm::BasicBlock::InstListType &instrs = bb_it->getInstList();
    //     auto instrs_it = instrs.begin();
    //     auto instrs_end = instrs.end();
    //     bool delete_the_rest = false;
    //     while (instrs_it != instrs_end)
    //     {
    //         if (delete_the_rest)
    //         {
    //             // instrs_it->eraseFromParent();
    //             llvm::Instruction *current_inst = (llvm::Instruction *)&*instrs_it;
    //             current_inst->eraseFromParent();
    //         }
    //         if (instrs_it->isTerminator())
    //         {
    //             cout << "term" << endl;
    //             delete_the_rest = true;
    //         }
    //         instrs_it++;
    //     }

    //     bb_it++;
    // }

    if (!entry->getTerminator())
    {
        builder.CreateRetVoid();
    }

    if (compiler_options.optimize)
        fpm->run(*function);

    // if (verifyFunction(*function, &outs()))
    // {
    // print_current_module();
    // exit(1);
    // }

    return function;
}

void Prototype_Node::create_argument_allocas(Function *function)
{
    Function::arg_iterator it = function->arg_begin();
    for (unsigned i = 0, size = param_names.size(); i != size; ++i, ++it)
    {
        auto ptr = create_entry_block_alloca(function, param_names[i], variable_type_to_llvm_type(param_types[i], parameter_type_names[i]));
        auto store = builder.CreateStore(it, ptr);
        auto loaded = builder.CreateLoad(ptr, ptr->getName() + "_loaded");
        functions[current_function_name]->set_variable_ptr(param_names[i], ptr);
        functions[current_function_name]->set_variable_value(param_names[i], loaded);
        //! this might be broken because i is not the correct index for parameter_type_names
        functions[current_function_name]->set_variable_type_names(param_names[i], parameter_type_names[i]);
    }
}

Value *create_entry_block_alloca(Function *function, const std::string &name, Type *type)
{
    IRBuilder<> tmp_builder(&function->getEntryBlock(), function->getEntryBlock().begin());
    return tmp_builder.CreateAlloca(type, 0, name.c_str());
}

Function *Prototype_Node::code_gen_proto()
{
    std::vector<Type *> types;
    std::map<int, llvm::Type *> dereferenced_types;
    int i = 0;
    for (auto &param_type : param_types)
    {
        llvm::Type *llvm_type;
        if (param_type == Variable_Type::type_object || param_type == Variable_Type::type_object_ptr || param_type == Variable_Type::type_object_ref)
        {
            // cout << "Hi" << endl;
            auto parameter_type_names = get_parameter_type_names();
            // cout << "Hi" << endl;
            llvm_type = variable_type_to_llvm_type(param_type, parameter_type_names[i]);
            // cout << "Hi" << endl;
            // print_t(llvm_type);
        }
        else
        {
            llvm_type = variable_type_to_llvm_type(param_type);
        }

        if (is_reference_type(param_type))
            dereferenced_types[i] = llvm_type;
        types.push_back(llvm_type);
        i++;
    }
    Type *ret;
    if (return_type == Variable_Type::type_object)
        ret = variable_type_to_llvm_type(return_type, return_type_name);
    else
        ret = variable_type_to_llvm_type(return_type);

    llvm::FunctionType *function_type = llvm::FunctionType::get(ret, types, false);
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, name, modules[current_module_pointer].get());

    auto dl = new llvm::DataLayout(modules[current_module_pointer].get());
    auto it = dereferenced_types.begin();
    while (it != dereferenced_types.end())
    {
        unsigned bytes = dl->getTypeAllocSize(it->second);
        f->addDereferenceableAttr(it->first + 1, bytes);
        it++;
    }

    if (f->getName() != name)
    {
        f->eraseFromParent();
        f = modules[current_module_pointer]->getFunction(name);

        if (!f->empty())
            error("Redefinition of function");

        if (f->arg_size() != param_names.size())
            error("Redefinition of function with different number of arguments");
    }

    unsigned idx = 0;
    for (llvm::Function::arg_iterator ai = f->arg_begin(); idx != param_names.size(); ai++, idx++)
    {
        ai->setName(param_names[idx]);
    }

    return f;
}

Value *Expression_Node::code_gen()
{
    return 0;
}
Value *Binary_Operation_Expression_Node::code_gen()
{
    if (op == "=")
    {
        auto l = static_cast<Variable_Expression_Node *>(lhs.get());
        if (l)
        {
            auto var = functions[current_function_name]->get_variable_ptr(l->get_name());
            if (!var)
                error("Unknown variable referenced");

            auto val = rhs->code_gen();

            return builder.CreateStore(val, var);
        }
        else
        {
            auto l = static_cast<Object_Property_Assignment_Node *>(lhs.get());
            if (l)
            {
                auto ptr = l->code_gen();
                print_v(ptr);
                auto v = rhs->code_gen();
                print_v(v);
                builder.CreateStore(v, ptr);
                // auto var = functions[current_function_name]->get_variable_ptr(l->get_object_name());
                // if (!var)
                //     error("Unknown variable referenced");
                // print_v(var);
            }
            else
            {
                error("Left hand side of '=' must be a variable");
            }
        }

        // if (!l)
        // {
        //     l = static_cast<Object_Property_Assignment_Node *>(lhs.get());
        //     if (!l)
        //         error("Left hand side of '=' must be a variable");
        // }
    }

    auto l = lhs->code_gen();
    auto r = rhs->code_gen();

    if (l == 0 || r == 0)
    {
        error("Error in binary operation codegen");
        return 0;
    }

    auto l_loaded = load_if_ptr(l);
    auto r_loaded = load_if_ptr(r);

    auto l_type = l_loaded->getType();
    auto r_type = r_loaded->getType();

    if (l_type->isDoubleTy() || l_type->isFloatTy() || r_type->isDoubleTy() || r_type->isFloatTy())
    {
        if (op == "+")
            return builder.CreateFAdd(l_loaded, r_loaded, "addtmp");
        if (op == "-")
            return builder.CreateFSub(l_loaded, r_loaded, "subtmp");
        if (op == "*")
            return builder.CreateFMul(l_loaded, r_loaded, "multmp");
        if (op == "<")
            return builder.CreateFCmpOLT(l_loaded, r_loaded, "lttmp");
    }
    else
    {
        if (op == "+")
            return builder.CreateAdd(l_loaded, r_loaded, "addtmp");
        if (op == "-")
            return builder.CreateSub(l_loaded, r_loaded, "subtmp");
        if (op == "*")
            return builder.CreateMul(l_loaded, r_loaded, "multmp");
        if (op == "<")
            return builder.CreateICmpSLT(l_loaded, r_loaded, "lttmp");
    }

    return 0;
}

Value *Number_Expression_Node::code_gen()
{
    // cout << "NUMBER: " << variable_type << ' ' << currently_preferred_type << endl;
    if (variable_type == Variable_Type::type_null)
        variable_type = currently_preferred_type;
    switch (variable_type)
    {
    case Variable_Type::type_i64:
        return llvm::ConstantInt::get(context, llvm::APInt(64, (int)value, true));
    case Variable_Type::type_i32:
        return llvm::ConstantInt::get(context, llvm::APInt(32, (int)value, true));
    case Variable_Type::type_i16:
        return llvm::ConstantInt::get(context, llvm::APInt(16, (int)value, true));
    case Variable_Type::type_i8:
        return llvm::ConstantInt::get(context, llvm::APInt(8, (int)value, true));
    case Variable_Type::type_float:
        return llvm::ConstantFP::get(context, llvm::APFloat((float)value));
    case Variable_Type::type_double:
        return llvm::ConstantFP::get(context, llvm::APFloat((double)value));
    case Variable_Type::type_bool:
        return llvm::ConstantInt::get(context, llvm::APInt(1, (int)value, true));
    default:
        error("Could not codegen number");
        return nullptr;
    }
}

Value *Prototype_Node::code_gen()
{
    return 0;
}
Value *Then_Node::code_gen()
{
    for (auto &node : nodes)
    {
        node->code_gen();
    }
    return 0;
}
Value *Variable_Declaration_Node::code_gen()
{
    if (type == Variable_Type::type_object)
    {
        auto llvm_ty = object_types[type_name];
        auto ptr = builder.CreateAlloca(llvm_ty, 0, name);
        functions[current_function_name]->set_variable_ptr(name, ptr);
        functions[current_function_name]->set_variable_type_names(name, type_name);
        if (undefined)
            return ptr;
        auto fn_call = static_cast<Function_Call_Node *>(std::move(value).get());
        if (fn_call)
        {
            auto v = fn_call->code_gen();
            builder.CreateStore(v, ptr);
        }
        else
        {
            define_object_properties(this, ptr, std::move(value));
        }

        return 0;
    }
    else if (type == Variable_Type::type_array)
        return code_gen_array_variable_declaration(this);
    else if (type == Variable_Type::type_string)
        return code_gen_string_variable_declaration(this);
    else
        return code_gen_primitive_variable_declaration(this);
}

Value *initialize_string(String_Expression *)
{
    return 0;
}

Value *code_gen_string_variable_declaration(Variable_Declaration_Node *var)
{
    auto string_expr = static_cast<String_Expression *>(var->get_value().get());

    auto llvm_ty = object_types["string"];
    auto ptr = builder.CreateAlloca(llvm_ty, 0, var->get_name());
    auto fn_call = static_cast<Function_Call_Node *>(std::move(var->get_value()).get());
    if (fn_call)
    {
        auto v = fn_call->code_gen();
        builder.CreateStore(v, ptr);
    }
    else
    {
        cout << "not fn call" << endl;
        cout << "size: " << string_expr->get_value().size() << endl;
    }

    // functions[current_function_name]->set_variable_ptr(name, ptr);
    // auto string_val = string_expr->get_value();
    // auto string_len = string_val.size();
    // cout << string_val << endl;

    // auto array_type = llvm::ArrayType::get(llvm::Type::getInt8Ty(context), string_len + 2);
    // auto ptr = builder.CreateAlloca(array_type, 0, var->get_name());

    // int i = 0;
    // for (auto &c : string_val)
    // {
    //     auto gep = builder.CreateStructGEP(ptr, i);
    //     auto store = builder.CreateStore(llvm::ConstantInt::get(llvm::Type::getInt8Ty(context), llvm::APInt(8, c)), gep);
    //     i++;
    // }

    // auto nlgep = builder.CreateStructGEP(ptr, i);
    // builder.CreateStore(llvm::ConstantInt::get(llvm::Type::getInt8Ty(context), llvm::APInt(8, 12)), nlgep);
    // i++;
    // auto nullgep = builder.CreateStructGEP(ptr, i);
    // builder.CreateStore(llvm::ConstantInt::get(llvm::Type::getInt8Ty(context), llvm::APInt(8, 00)), nullgep);

    // functions[current_function_name]->set_variable_ptr(var->get_name(), ptr);

    return 0;
}

Value *code_gen_array_variable_declaration(Variable_Declaration_Node *var)
{
    currently_preferred_type = var->get_array_type();
    auto members_type = variable_type_to_llvm_type(var->get_array_type());
    auto members = var->get_value()->get_members();

    auto tst = static_cast<Array_Expression *>(var->get_value().get());

    auto array_type = llvm::ArrayType::get(members_type, members.size());
    auto ptr = builder.CreateAlloca(array_type, 0, var->get_name());

    int i = 0;
    for (auto &member : members)
    {
        auto gep = builder.CreateStructGEP(ptr, i);
        auto store = builder.CreateStore(member->code_gen(), gep);
        i++;
    }

    currently_preferred_type = Variable_Type::type_i32;

    return 0;
}

void define_object_properties(Variable_Declaration_Node *var, Value *ptr, unique_ptr<Expression_Node> expr)
{
    auto properties = expr->get_properties();
    auto it = properties.begin();
    for (unsigned i = 0; it != properties.end(); i++)
    {
        auto variable_type = object_type_properties[var->get_type_name()][it->first];
        currently_preferred_type = variable_type;

        auto index = APInt(32, i);
        auto index_value = Constant::getIntegerValue(Type::getInt32Ty(context), index);

        auto v = load_if_ptr(it->second->code_gen());
        auto val_ptr = builder.CreateStructGEP(ptr, i, it->first + "_ptr");
        auto store = builder.CreateStore(v, val_ptr);

        it++;
    }
}

Value *code_gen_primitive_variable_declaration(Variable_Declaration_Node *var)
{
    auto llvm_ty = variable_type_to_llvm_type(var->get_type());
    auto ptr = builder.CreateAlloca(llvm_ty, 0, var->get_name());

    if (var->is_undefined())
    {
        functions[current_function_name]->set_variable_ptr(var->get_name(), ptr);
        return ptr;
    }

    auto v = var->get_value()->code_gen();
    auto store = builder.CreateStore(v, ptr);
    if (var->get_type() == Variable_Type::type_i32_ptr)
    {
        functions[current_function_name]->set_variable_ptr(var->get_name(), ptr);
        return v;
    }

    functions[current_function_name]->set_variable_ptr(var->get_name(), ptr);

    return ptr;
}

Value *If_Node::code_gen()
{
    std::vector<Value *> cmps;
    for (auto &condition : conditions)
    {
        auto cmp = condition->code_gen();
        cmps.push_back(cmp);
    }

    Value *last_cmp = cmps[0];
    int i = 0;
    for (auto &sep : condition_separators)
    {
        switch (sep)
        {
        case Token_Type::tok_and:
            if (i == 0)
                last_cmp = builder.CreateAnd(cmps[i], cmps[i + 1]);
            else
                last_cmp = builder.CreateAnd(last_cmp, cmps[i + 1]);
            break;
        case Token_Type::tok_or:
        {
            if (i == 0)
                last_cmp = builder.CreateOr(cmps[i], cmps[i + 1]);
            else
                last_cmp = builder.CreateOr(last_cmp, cmps[i + 1]);
        }
        default:
            break;
        }
        i++;
    }

    auto function = builder.GetInsertBlock()->getParent();

    auto then_bb = BasicBlock::Create(context, "then", function);
    auto else_bb = BasicBlock::Create(context, "else");
    auto continue_bb = BasicBlock::Create(context, "continue");

    auto cond_br = builder.CreateCondBr(last_cmp, then_bb, else_bb);

    builder.SetInsertPoint(then_bb);

    then->code_gen();

    builder.CreateBr(continue_bb);

    then_bb = builder.GetInsertBlock();

    function->getBasicBlockList().push_back(else_bb);
    builder.SetInsertPoint(else_bb);

    builder.CreateBr(continue_bb);

    else_bb = builder.GetInsertBlock();

    function->getBasicBlockList().push_back(continue_bb);
    builder.SetInsertPoint(continue_bb);

    return 0;
}
Value *For_Node::code_gen()
{
    auto the_function = builder.GetInsertBlock()->getParent();
    auto var = variable->code_gen();
    auto cond = condition->code_gen();
    auto loop_bb = llvm::BasicBlock::Create(context, "loop", the_function);
    auto continue_bb = llvm::BasicBlock::Create(context, "continue", the_function);
    auto br = builder.CreateCondBr(cond, loop_bb, continue_bb);
    builder.SetInsertPoint(loop_bb);
    then->code_gen();
    assignment->code_gen();

    auto new_cond = condition->code_gen();
    auto repeat = builder.CreateCondBr(new_cond, loop_bb, continue_bb);

    builder.SetInsertPoint(continue_bb);

    return 0;
}
Value *Condition_Node::code_gen()
{
    currently_preferred_type = Variable_Type::type_i32;
    auto l = lhs->code_gen();
    currently_preferred_type = llvm_type_to_variable_type(l->getType());
    auto r = rhs->code_gen();

    switch (op)
    {
    case Token_Type::tok_compare_eq:
        return builder.CreateICmpEQ(l, r, "ifcond");
    case Token_Type::tok_compare_lt:
        return builder.CreateICmpSLT(l, r, "ltcond");
    case Token_Type::tok_compare_gt:
        return builder.CreateICmpSGT(l, r, "gtcond");
    default:
        error("Unknown operator in condition");
        return 0;
    }
    currently_preferred_type = Variable_Type::type_bool;
}
Value *Function_Call_Node::code_gen()
{
    auto callee_function = modules[current_module_pointer]->getFunction(name);

    // cout << callee_function->arg_size() << ' ' << parameters.size() << endl;
    // cout << callee_function->isVarArg() << endl;

    if (callee_function == 0)
        error("Unknown function referenced in function call");
    if (callee_function->arg_size() != parameters.size() && !callee_function->isVarArg())
        error("Incorrect number of parameters passed to function call");

    std::vector<llvm::Type *> arg_types;
    auto args = callee_function->args();
    auto it = args.begin();
    while (it != args.end())
    {
        arg_types.push_back(it->getType());
        it++;
    }

    std::vector<llvm::Value *> args_v;
    std::map<int, uint64_t> dereferenced_types;
    for (unsigned i = 0, size = parameters.size(); i != size; i++)
    {
        auto bytes = callee_function->getParamDereferenceableBytes(i);
        llvm::Value *v;

        if (bytes)
        {
            wants_reference = true;
            v = parameters[i]->code_gen();
            dereferenced_types[i] = bytes;
            args_v.push_back(v);
            wants_reference = false;
        }
        else
        {
            // cout << "arg type: ";
            // print_t(arg_types[i]);
            if (i < arg_types.size())
                currently_preferred_type = llvm_type_to_variable_type(arg_types[i]);
            // cout << "preffered: " << currently_preferred_type << endl;
            v = parameters[i]->code_gen();
            args_v.push_back(v);
            currently_preferred_type = llvm_type_to_variable_type(builder.GetInsertBlock()->getParent()->getReturnType());
        }
    }

    auto call = builder.CreateCall(callee_function, args_v, "calltmp");

    auto dereferenced_it = dereferenced_types.begin();
    while (dereferenced_it != dereferenced_types.end())
    {
        call->addDereferenceableAttr(dereferenced_it->first + 1, dereferenced_it->second);
        dereferenced_it++;
    }

    return call;
}
Value *Variable_Expression_Node::code_gen()
{
    if (functions[current_function_name]->get_variable_type_names(name).size() > 0)
    {
        // ! JESUSSSSS -- ok, we need to check if it's looking for a pointer or something. The problem is this god damn Object_Property_Assign thing
        //! It's stupid, just make '.' an operator so it can be an Expression focus on this tmrw
        // if()
    }
    if (wants_reference)
        return functions[current_function_name]->get_variable_ptr(name);
    if (type == Variable_Expression_Type::reference)
        return functions[current_function_name]->get_variable_ptr(name);
    else if (type == Variable_Expression_Type::pointer)
    {
        auto ptr = builder.CreateLoad(functions[current_function_name]->get_variable_ptr(name), functions[current_function_name]->get_variable_ptr(name)->getName() + "_loaded");
        auto val = builder.CreateLoad(ptr, ptr->getName() + "_loaded");
        return val;
    }
    else
    {
        auto ptr = functions[current_function_name]->get_variable_ptr(name);
        auto ty = ptr->getType();
        if (ty->isPointerTy())
        {
            auto contained_type = ty->getContainedType(0);
            if (contained_type->isArrayTy())
            {
                cout << "it's array" << endl;
                return builder.CreateStructGEP(ptr, 0, ptr->getName() + "_loaded");
            }
        }
        return builder.CreateLoad(ptr, ptr->getName() + "_loaded");
    }
}
Value *Import_Node::code_gen()
{
    return 0;
}

Value *Object_Node::code_gen()
{
    auto it = properties.begin();

    std::vector<llvm::Type *> members;
    while (it != properties.end())
    {
        auto ty = variable_type_to_llvm_type(it->second);
        members.push_back(ty);
        object_type_properties[name][it->first] = it->second;

        it++;
    }

    llvm::ArrayRef<llvm::Type *> struct_properties(members);
    auto struct_type = llvm::StructType::create(context, struct_properties, name, false);

    object_types[name] = struct_type;

    return 0;
}
Value *Object_Expression::code_gen()
{

    return 0;
}
Value *String_Expression::code_gen()
{
    // auto string_len = value.size();

    // auto array_type = llvm::ArrayType::get(llvm::Type::getInt8Ty(context), string_len);
    // auto ptr = builder.CreateAlloca(array_type, 0, var->get_name());

    // int i = 0;
    // for (auto &c : string_val)
    // {
    //     auto gep = builder.CreateStructGEP(ptr, i);
    //     auto store = builder.CreateStore(llvm::ConstantInt::get(llvm::Type::getInt8Ty(context), llvm::APInt(8, c)), gep);
    //     i++;
    // }
    return 0;
}

Value *Return_Node::code_gen()
{
    auto v = value->code_gen();
    auto ret = builder.CreateRet(v);
    return ret;
}

Value *Array_Expression::code_gen()
{
    return 0;
}

Value *Object_Property_Assignment_Node::code_gen()
{
    auto var_type = functions[current_function_name]->get_variable_type_names(object_name);
    auto properties = object_type_properties[var_type];
    auto begin = properties.begin();
    auto end = properties.end();
    int i = 0;
    while (begin != end)
    {
        if (begin->first == property_name)
        {
            auto v = functions[current_function_name]->get_variable_ptr(object_name);
            if (has_asterisk)
                v = builder.CreateLoad(v);
            auto ptr = builder.CreateGEP(v, llvm::ConstantInt::get(llvm::Type::getInt32Ty(context), llvm::APInt(32, i)), v->getName() + "_" + begin->first);
            currently_preferred_type = begin->second;
            return ptr;
        }
        begin++;
        i++;
    }
    return 0;
}

Type *variable_type_to_llvm_type(Variable_Type type, std::string object_type_name)
{
    switch (type)
    {
    case Variable_Type::type_i64:
        return Type::getInt64Ty(context);
    case Variable_Type::type_i32:
        return Type::getInt32Ty(context);
    case Variable_Type::type_i16:
        return Type::getInt16Ty(context);
    case Variable_Type::type_i8:
        return Type::getInt8Ty(context);
    case Variable_Type::type_bool:
        return Type::getInt1Ty(context);
    case Variable_Type::type_float:
        return Type::getFloatTy(context);
    case Variable_Type::type_double:
        return Type::getDoubleTy(context);
    case Variable_Type::type_object:
        return object_types[object_type_name];
    case Variable_Type::type_i64_ptr:
        return Type::getInt64PtrTy(context);
    case Variable_Type::type_i32_ptr:
        return Type::getInt32PtrTy(context);
    case Variable_Type::type_i16_ptr:
        return Type::getInt16PtrTy(context);
    case Variable_Type::type_i8_ptr:
        return Type::getInt8PtrTy(context);
    case Variable_Type::type_bool_ptr:
        return Type::getInt1PtrTy(context);
    case Variable_Type::type_float_ptr:
        return Type::getFloatPtrTy(context);
    case Variable_Type::type_double_ptr:
        return Type::getDoublePtrTy(context);
    case Variable_Type::type_object_ptr:
    {
        // ! WHY THE FUCK CAN'T I JUST DO OBJECT_TYPES[OBJECT_TYPE_NAME] ????? WHAT?
        auto it = object_types.begin();
        while (it != object_types.end())
        {
            if (it->first == object_type_name)
                return llvm::PointerType::get(it->second, 0);
            it++;
        }
    }
    case Variable_Type::type_i64_ref:
        return Type::getInt64PtrTy(context);
    case Variable_Type::type_i32_ref:
        return Type::getInt32PtrTy(context);
    case Variable_Type::type_i16_ref:
        return Type::getInt16PtrTy(context);
    case Variable_Type::type_i8_ref:
        return Type::getInt8PtrTy(context);
    case Variable_Type::type_bool_ref:
        return Type::getInt1PtrTy(context);
    case Variable_Type::type_float_ref:
        return Type::getFloatPtrTy(context);
    case Variable_Type::type_double_ref:
        return Type::getDoublePtrTy(context);
    case Variable_Type::type_void:
        return Type::getVoidTy(context);
    default:
        error("Could not convert variable type to llvm type");
        return nullptr;
    }
}

bool is_reference_type(Variable_Type type)
{
    switch (type)
    {
    case Variable_Type::type_i64_ref:
        return true;
    case Variable_Type::type_i32_ref:
        return true;
    case Variable_Type::type_i16_ref:
        return true;
    case Variable_Type::type_i8_ref:
        return true;
    case Variable_Type::type_bool_ref:
        return true;
    case Variable_Type::type_float_ref:
        return true;
    case Variable_Type::type_double_ref:
        return true;
    default:
        return false;
    }
}

Variable_Type llvm_type_to_variable_type(llvm::Type *type)
{
    if (type->isIntegerTy(64))
    {
        return Variable_Type::type_i64;
    }
    else if (type->isIntegerTy(32))
    {
        return Variable_Type::type_i32;
    }
    else if (type->isIntegerTy(16))
    {
        return Variable_Type::type_i16;
    }
    else if (type->isIntegerTy(8))
    {
        return Variable_Type::type_i8;
    }
    else if (type->isIntegerTy(1))
    {
        return Variable_Type::type_bool;
    }
    else if (type->isIntOrPtrTy())
    {
        return Variable_Type::type_i8_ptr;
    }
    else if (type->isFloatTy())
    {
        return Variable_Type::type_float;
    }
    else if (type->isDoubleTy())
    {
        return Variable_Type::type_double;
    }

    error("Could not convert llvm type to variable type");
    return Variable_Type::type_null;
}

Value *load_if_ptr(Value *v)
{
    if (v->getType()->isPointerTy())
        return builder.CreateLoad(v, v->getName() + "_loaded");
    return v;
}

void print_current_module()
{
    std::error_code ec;
    auto f_out = raw_fd_ostream("../out.ll", ec);
    llvm::raw_ostream &os = llvm::outs();
    os << '\n'
       << '\n';
    auto writer = new AssemblyAnnotationWriter();
    modules[current_module_pointer]->print(os, writer);
    modules[current_module_pointer]->print(f_out, writer);
}

void error(const char *arg)
{
    cout << arg << endl;

    print_current_module();

    exit(1);
}