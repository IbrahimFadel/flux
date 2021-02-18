#include "code_generation.h"

void create_module(const Nodes &nodes, CompilerOptions options, std::string path, Dependency_Tree *tree)
{
    module = new llvm::Module(path, context);
    compiler_options = options;

    declare_c_functions(module);
    declare_imported_functions(tree, fs::canonical(path), module);

    if (compiler_options.optimize)
        initialize_fpm(module);

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node), module);
    }

    if (compiler_options.print_module)
        print_module(module);

    write_module_to_file(module, "../main.ll");
}

void module_to_obj(llvm::Module *mod, std::string output_path)
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
        exit(1);
    }

    auto CPU = "generic";
    auto features = "";

    llvm::TargetOptions opt;
    auto rm = llvm::Optional<llvm::Reloc::Model>();
    auto target_machine = target->createTargetMachine(target_triple, CPU, features, opt, rm);

    mod->setDataLayout(target_machine->createDataLayout());
    mod->setTargetTriple(target_triple);

    std::error_code ec;
    llvm::raw_fd_ostream dest(output_path, ec, llvm::sys::fs::F_None);
    if (ec)
    {
        llvm::errs() << "Could not open file: " << ec.message();
        exit(1);
    }

    llvm::legacy::PassManager pass;
    auto file_type = llvm::CGFT_ObjectFile;

    if (target_machine->addPassesToEmitFile(pass, dest, nullptr, file_type))
    {
        llvm::errs() << "Target Machine cannot emit a file of this type";
        exit(1);
    }

    pass.run(*mod);
    dest.flush();
}

void declare_c_functions(llvm::Module *mod)
{
    std::vector<llvm::Type *> free_param_types;
    free_param_types.push_back(llvm::Type::getInt8PtrTy(context));
    auto free_return_type = llvm::Type::getVoidTy(context);
    declare_function("free", free_param_types, free_return_type, mod);

    std::vector<llvm::Type *> malloc_param_types;
    malloc_param_types.push_back(llvm::Type::getInt64Ty(context));
    auto malloc_return_type = llvm::Type::getInt8PtrTy(context);
    declare_function("malloc", malloc_param_types, malloc_return_type, mod);

    std::vector<llvm::Type *> memcpy_param_types;
    memcpy_param_types.push_back(llvm::Type::getInt8PtrTy(context));
    memcpy_param_types.push_back(llvm::Type::getInt8PtrTy(context));
    memcpy_param_types.push_back(llvm::Type::getInt64Ty(context));
    auto memcpy_return_type = llvm::Type::getInt8PtrTy(context);
    declare_function("memcpy", memcpy_param_types, memcpy_return_type, mod);

    std::vector<llvm::Type *> printf_param_types;
    printf_param_types.push_back(llvm::Type::getInt8PtrTy(context));
    auto printf_return_type = llvm::Type::getInt32Ty(context);
    declare_function("printf", printf_param_types, printf_return_type, mod);
}

void declare_imported_functions(Dependency_Tree *tree, fs::path path, llvm::Module *mod)
{
    for (auto &connection : tree->connections)
    {
        auto left_path = tree->nodes[connection.first].first;
        if (left_path == path)
        {
            auto funcs_to_declare = tree->nodes[connection.second].second;
            for (auto &func : funcs_to_declare)
            {
                std::vector<llvm::Type *> param_types;
                for (auto &param_type : func->param_types)
                {
                    param_types.push_back(ss_type_to_llvm_type(param_type));
                }
                declare_function(func->name, param_types, ss_type_to_llvm_type(func->return_type), mod);
            }
        }
    }
}

void code_gen_node(const unique_ptr<Node> &node, llvm::Module *mod)
{
    node->code_gen(mod);
}

void initialize_fpm(llvm::Module *mod)
{
    fpm = std::make_unique<llvm::legacy::FunctionPassManager>(mod);
    fpm->add(llvm::createInstructionCombiningPass());
    fpm->add(llvm::createReassociatePass());
    fpm->add(llvm::createDeadCodeEliminationPass());
    fpm->add(llvm::createGVNPass());
    fpm->add(llvm::createCFGSimplificationPass());
    fpm->add(llvm::createPromoteMemoryToRegisterPass());

    fpm->doInitialization();
}

llvm::Value *Function_Declaration::code_gen(llvm::Module *mod)
{
    llvm::Function *f = code_gen_function_prototype(params, return_type, name, mod);
    if (f == 0)
        return 0;

    auto entry = llvm::BasicBlock::Create(context, "entry", f);
    builder.SetInsertPoint(entry);

    current_function_name = name;
    functions[current_function_name] = this;

    create_function_param_allocas(f, params);

    currently_preferred_type = ss_type_to_llvm_type(return_type);

    then->code_gen(mod);

    auto last_bb = builder.GetInsertBlock();

    if (!last_bb->getTerminator())
    {
        builder.CreateRetVoid();
    }

    llvm::verifyFunction(*f, &llvm::outs());

    if (compiler_options.optimize)
        fpm->run(*f);

    return 0;
}

llvm::Function *code_gen_function_prototype(std::map<std::string, std::string> params, std::string return_type_str, std::string function_name, llvm::Module *mod)
{
    std::vector<std::string> param_names;
    std::vector<llvm::Type *> param_types;
    auto it = params.begin();
    while (it != params.end())
    {
        param_names.push_back(it->first);
        auto type = ss_type_to_llvm_type(it->second);
        param_types.push_back(type);
        it++;
    }

    auto return_type = ss_type_to_llvm_type(return_type_str);
    llvm::FunctionType *function_type = llvm::FunctionType::get(return_type, param_types, false);
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, function_name, mod);

    if (f->getName() != function_name)
    {
        f->eraseFromParent();
        f = mod->getFunction(function_name);

        if (!f->empty())
            fatal_error("Redefinition of function " + function_name);

        if (f->arg_size() != params.size())
            fatal_error("Redefinition of function with different number of arguments");
    }

    unsigned idx = 0;
    for (llvm::Function::arg_iterator ai = f->arg_begin(); idx != params.size(); ai++, idx++)
    {
        ai->setName(param_names[idx]);
    }

    return f;
}

llvm::Value *Number_Expression::code_gen(llvm::Module *mod)
{
    return llvm::ConstantInt::get(currently_preferred_type, value);
}

llvm::Value *Variable_Reference_Expression::code_gen(llvm::Module *mod)
{
    return functions[current_function_name]->get_variable(name);
}

llvm::Value *Index_Accessed_Expression::code_gen(llvm::Module *mod)
{
    auto var = expr->code_gen(mod);
    auto index_v = index->code_gen(mod);
    auto gep = builder.CreateGEP(var, index_v);
    auto loaded = builder.CreateLoad(gep);
    return loaded;
}

llvm::Value *code_gen_binop_sum_diff_prod_quot(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, llvm::Module *mod)
{
    auto l = lhs->code_gen(mod);
    auto r = rhs->code_gen(mod);
    auto sum = builder.CreateAdd(l, r);
    return sum;
}

llvm::Value *code_gen_binop_eq(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, llvm::Module *mod)
{
    auto l = lhs->code_gen(mod);
    currently_preferred_type = l->getType();
    auto r = rhs->code_gen(mod);
    auto l_ptr = llvm::getPointerOperand(l);
    auto store = builder.CreateStore(r, l_ptr);
    return store;
}

llvm::Value *code_gen_binop_arrow(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, llvm::Module *mod)
{
    auto lhs_var_reference_expr = dynamic_cast<Variable_Reference_Expression *>(lhs.get());
    if (lhs_var_reference_expr == nullptr)
        fatal_error("Object property assignments must have a variable reference on the left hand side");

    auto rhs_var_reference_expr = dynamic_cast<Variable_Reference_Expression *>(rhs.get());
    if (rhs_var_reference_expr == nullptr)
        fatal_error("Object property assignments must have a variable reference on the right hand side");

    auto prop_name = rhs_var_reference_expr->get_name();

    auto l_var = lhs_var_reference_expr->code_gen(mod);
    auto ty = l_var->getType();
    std::string struct_name;
    while (ty->isPointerTy())
        ty = ty->getPointerElementType();
    struct_name = ty->getStructName().str();

    int i = 0;
    int prop_index = -1;
    auto property_insertion_order = struct_property_insertion_orders[struct_name];
    auto properties = struct_properties[struct_name];
    for (auto &prop : property_insertion_order)
    {
        if (prop == prop_name)
            prop_index = i;
        i++;
    }

    if (prop_index == -1)
        fatal_error("Could not find property in struct");
    auto gep = builder.CreateStructGEP(l_var, prop_index, prop_name);
    auto loaded = builder.CreateLoad(gep);
    return loaded;
}

llvm::Value *code_gen_binop_period(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, llvm::Module *mod)
{
    auto lhs_var_reference_expr = dynamic_cast<Variable_Reference_Expression *>(lhs.get());
    if (lhs_var_reference_expr == nullptr)
        fatal_error("Object property assignments must have a variable reference on the left hand side");

    auto rhs_var_reference_expr = dynamic_cast<Variable_Reference_Expression *>(rhs.get());
    if (rhs_var_reference_expr == nullptr)
        fatal_error("Object property assignments must have a variable reference on the right hand side");

    auto prop_name = rhs_var_reference_expr->get_name();

    auto l_var = lhs_var_reference_expr->code_gen(mod);
    auto ty = l_var->getType();
    std::string struct_name;
    while (ty->isPointerTy())
        ty = ty->getPointerElementType();
    struct_name = ty->getStructName().str();

    int i = 0;
    int prop_index = -1;
    auto property_insertion_order = struct_property_insertion_orders[struct_name];
    auto properties = struct_properties[struct_name];
    for (auto &prop : property_insertion_order)
    {
        if (prop == prop_name)
            prop_index = i;
        i++;
    }

    if (prop_index == -1)
        fatal_error("Could not find property in struct");

    auto l_var_ptr = llvm::getPointerOperand(l_var);
    auto gep = builder.CreateStructGEP(l_var_ptr, prop_index, prop_name);
    auto loaded = builder.CreateLoad(gep);
    return loaded;
}

llvm::Value *code_gen_binop_cmp(const unique_ptr<Expression> &lhs, const unique_ptr<Expression> &rhs, Token_Type op, llvm::Module *mod)
{
    auto l = lhs->code_gen(mod);
    auto r = rhs->code_gen(mod);

    switch (op)
    {
    case Token_Type::tok_compare_eq:
        return builder.CreateICmpEQ(l, r);
    case Token_Type::tok_compare_ne:
        return builder.CreateICmpNE(l, r);
    case Token_Type::tok_compare_lt:
        return builder.CreateICmpULT(l, r);
    case Token_Type::tok_compare_gt:
        return builder.CreateICmpUGT(l, r);
    default:
        break;
    }
    return 0;
}

llvm::Value *Binary_Operation_Expression::code_gen(llvm::Module *mod)
{
    switch (op)
    {
    case Token_Type::tok_plus:
        return code_gen_binop_sum_diff_prod_quot(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_minus:
        return code_gen_binop_sum_diff_prod_quot(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_asterisk:
        return code_gen_binop_sum_diff_prod_quot(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_slash:
        return code_gen_binop_sum_diff_prod_quot(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_eq:
        return code_gen_binop_eq(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_arrow:
        return code_gen_binop_arrow(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_period:
        return code_gen_binop_period(std::move(lhs), std::move(rhs), mod);
    case Token_Type::tok_compare_eq:
        return code_gen_binop_cmp(std::move(lhs), std::move(rhs), op, mod);
    case Token_Type::tok_compare_ne:
        return code_gen_binop_cmp(std::move(lhs), std::move(rhs), op, mod);
    default:
        fatal_error("Tried to codegen binary operation expression of unimplemented operator");
    }
    return 0;
}

llvm::Value *Nullptr_Expression::code_gen(llvm::Module *mod)
{
    while (currently_preferred_type->isPointerTy())
    {
        currently_preferred_type = currently_preferred_type->getPointerElementType();
    }
    return llvm::ConstantPointerNull::get(currently_preferred_type->getPointerTo());
}

llvm::Value *Unary_Prefix_Operation_Expression::code_gen(llvm::Module *mod)
{
    auto val = value->code_gen(mod);
    if (op == Token_Type::tok_ampersand)
    {
        return llvm::getPointerOperand(val);
    }
    else if (op == Token_Type::tok_asterisk)
    {
        return builder.CreateLoad(val);
    }
    fatal_error("Tried to codegen unary prefix opepration expression that is unimplemented");
    return 0;
}

llvm::Value *Code_Block::code_gen(llvm::Module *mod)
{
    for (auto &node : nodes)
    {
        node->code_gen(mod);
    }
    return 0;
}

llvm::Value *Variable_Declaration::code_gen(llvm::Module *mod)
{
    if (is_struct)
    {
        auto struct_val_expr = dynamic_cast<Struct_Value_Expression *>(value.get());
        if (struct_val_expr != nullptr)
        {
            return code_gen_struct_variable_declaration(name, type, struct_val_expr, mod);
        }
    }

    auto llvm_type = ss_type_to_llvm_type(type);
    auto ptr = builder.CreateAlloca(llvm_type, 0, name);

    if (value != nullptr)
    {
        currently_preferred_type = llvm_type;
        auto val = value->code_gen(mod);
        auto store = builder.CreateStore(val, ptr);
    }

    auto loaded = builder.CreateLoad(ptr, 0, name);
    functions[current_function_name]->set_variable(name, std::move(loaded));
    return loaded;
}

llvm::Value *code_gen_struct_variable_declaration(std::string name, std::string type, Struct_Value_Expression *value, llvm::Module *mod)
{
    auto ptr = builder.CreateAlloca(llvm_struct_types[type]);
    auto props = value->get_properties();
    int i = 0;
    for (auto &prop_name : value->get_property_insertion_order())
    {
        auto struct_property_ptr = builder.CreateStructGEP(ptr, i, prop_name);
        auto prop_ty = ss_type_to_llvm_type(struct_properties[type][prop_name]);
        currently_preferred_type = prop_ty;
        auto v = props[prop_name]->code_gen(mod);
        builder.CreateStore(v, struct_property_ptr);
        i++;
    }

    auto loaded = builder.CreateLoad(ptr);

    functions[current_function_name]->set_variable(name, std::move(loaded));

    return 0;
}

llvm::Value *Struct_Type_Expression::code_gen(llvm::Module *mod)
{
    std::vector<llvm::Type *> llvm_types;
    for (auto &prop_name : property_insetion_order)
    {
        auto ty = ss_type_to_llvm_type(properties[prop_name]);
        llvm_types.push_back(ty);
    }

    auto struct_type = llvm::StructType::create(context, llvm_types, name, false);

    llvm_struct_types[name] = struct_type;
    struct_properties[name] = properties;
    struct_property_insertion_orders[name] = property_insetion_order;

    return 0;
}

llvm::Value *Struct_Value_Expression::code_gen(llvm::Module *mod)
{
    return 0;
}

llvm::Value *If_Statement::code_gen(llvm::Module *mod)
{
    std::vector<llvm::Value *> cond_vs;
    for (auto &cond : conditions)
    {
        auto v = cond->code_gen(mod);
        cond_vs.push_back(v);
    }
    for (auto &sep : condition_separators)
    {
        fatal_error("Condition seperators in if statements not implemented yet");
        //TODO unimplemented
    }

    auto func = builder.GetInsertBlock()->getParent();
    auto then_bb = llvm::BasicBlock::Create(context, "if.then", func);
    auto else_bb = llvm::BasicBlock::Create(context, "if.else");
    auto merge_bb = llvm::BasicBlock::Create(context, "if.merge");

    auto br = builder.CreateCondBr(cond_vs[0], then_bb, else_bb);
    builder.SetInsertPoint(then_bb);

    then->code_gen(mod);

    builder.CreateBr(merge_bb);

    then_bb = builder.GetInsertBlock();

    func->getBasicBlockList().push_back(else_bb);
    builder.SetInsertPoint(else_bb);

    builder.CreateBr(merge_bb);

    else_bb = builder.GetInsertBlock();

    func->getBasicBlockList().push_back(merge_bb);
    builder.SetInsertPoint(merge_bb);

    return 0;
}

llvm::Value *Return_Statement::code_gen(llvm::Module *mod)
{
    currently_preferred_type = ss_type_to_llvm_type(functions[current_function_name]->get_return_type());
    auto v = value->code_gen(mod);
    builder.CreateRet(v);
    return 0;
}

llvm::Value *Import_Statement::code_gen(llvm::Module *mod)
{
    return 0;
}

llvm::Value *Function_Call_Expression::code_gen(llvm::Module *mod)
{
    if (name.substr(0, 1) == "@")
    {
        name = name.substr(1, name.size() - 1);
    }
    auto callee_function = mod->getFunction(name);
    if (callee_function == 0)
        fatal_error("Unknown function referenced in function call");
    if (callee_function->arg_size() != params.size())
        fatal_error("Incorrect number of parameters passed to function call");

    std::vector<llvm::Value *> param_values;
    std::vector<llvm::Type *> param_types;
    for (auto it = callee_function->args().begin(), end = callee_function->args().end(); it != end; it++)
    {
        param_types.push_back(it->getType());
    }

    int i = 0;
    for (auto &param : params)
    {
        currently_preferred_type = param_types[i];
        auto v = param->code_gen(mod);
        param_values.push_back(v);
        i++;
    }

    return builder.CreateCall(callee_function, param_values);
}

void declare_function(std::string name, std::vector<llvm::Type *> param_types, llvm::Type *return_type, llvm::Module *mod)
{
    llvm::FunctionType *function_type = llvm::FunctionType::get(return_type, param_types, false);
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, name, mod);
}

void create_function_param_allocas(llvm::Function *f, std::map<std::string, std::string> params)
{

    llvm::Function::arg_iterator f_it = f->arg_begin();
    auto param_it = params.begin();
    while (param_it != params.end())
    {
        auto ptr = create_entry_block_alloca(f, param_it->first, ss_type_to_llvm_type(param_it->second));
        auto store = builder.CreateStore(f_it, ptr);
        auto loaded = builder.CreateLoad(ptr);
        functions[current_function_name]->set_variable(param_it->first, std::move(loaded));
        param_it++;
        f_it++;
    }
}

llvm::Value *create_entry_block_alloca(llvm::Function *function, const std::string &name, llvm::Type *type)
{
    llvm::IRBuilder<> tmp_builder(&function->getEntryBlock(), function->getEntryBlock().begin());
    return tmp_builder.CreateAlloca(type);
}

llvm::Type *ss_type_to_llvm_type(std::string type)
{
    std::string base_type = "";
    for (const char &c : type)
    {
        if (c != (const char &)"*" && c != (const char &)"&")
        {
            base_type += c;
        }
    }

    auto llvm_type = ss_base_type_to_llvm_type(base_type);

    std::string rest = type.substr(base_type.size(), type.size() - 1);

    for (auto &c : rest)
    {
        if (c == (char &)"*")
        {
            llvm_type = llvm_type->getPointerTo();
        }
    }

    return llvm_type;
}

llvm::Type *ss_base_type_to_llvm_type(std::string type)
{
    if (type == "i64")
        return llvm::Type::getInt64Ty(context);
    else if (type == "i32")
        return llvm::Type::getInt32Ty(context);
    else if (type == "i16")
        return llvm::Type::getInt16Ty(context);
    else if (type == "i8")
        return llvm::Type::getInt8Ty(context);
    else if (type == "void")
        return llvm::Type::getVoidTy(context);
    else
    {
        if (std::find(struct_types.begin(), struct_types.end(), type) != struct_types.end())
            return llvm_struct_types[type];
        fatal_error("Could not convert base type to llvm type");
        return nullptr;
    }
}

void print_t(llvm::Type *ty)
{
    ty->print(llvm::outs());
    llvm::outs() << '\n';
}

void print_v(llvm::Value *v)
{
    v->print(llvm::outs());
    llvm::outs() << '\n';
}

void print_module(llvm::Module *mod)
{
    auto writer = new llvm::AssemblyAnnotationWriter();
    mod->print(llvm::outs(), writer);
}

void write_module_to_file(llvm::Module *mod, std::string path)
{
    auto writer = new llvm::AssemblyAnnotationWriter();
    std::error_code ec;
    auto f_out = llvm::raw_fd_ostream(path, ec);
    mod->print(f_out, writer);
}

void fatal_error(std::string msg)
{
    cout << msg << endl;
    exit(1);
}