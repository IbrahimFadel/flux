#include "code_generation.h"

void create_module(const Nodes &nodes, CompilerOptions options, std::string path, Dependency_Tree *tree, llvm::Module *mod)
{
    compiler_options = options;

    declare_imported_functions(tree, fs::canonical(path), mod);

    if (compiler_options.optimize)
        initialize_fpm(mod);

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node), mod);
    }

    if (compiler_options.print_module)
        print_module(mod);
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

    if (!entry->getTerminator())
    {
        builder.CreateRetVoid();
    }

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

llvm::Value *Binary_Operation_Expression::code_gen(llvm::Module *mod)
{
    auto l = lhs->code_gen(mod);
    auto r = rhs->code_gen(mod);

    switch (op)
    {
    case Token_Type::tok_plus:
        return builder.CreateAdd(l, r);
    default:
        fatal_error("Tried to codegen binary operation expression of unimplemented operator");
    }

    return 0;
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
    auto llvm_type = ss_type_to_llvm_type(type);
    auto ptr = builder.CreateAlloca(llvm_type, 0, name);

    if (value == nullptr)
    {
        functions[current_function_name]->set_variable(name, std::move(ptr));
        return ptr;
    }

    currently_preferred_type = llvm_type;
    auto val = value->code_gen(mod);
    auto store = builder.CreateStore(val, ptr);
    auto loaded = builder.CreateLoad(ptr, 0, name);
    functions[current_function_name]->set_variable(name, std::move(loaded));
    return loaded;
}

llvm::Value *Object_Type_Expression::code_gen(llvm::Module *mod)
{
    return 0;
}

llvm::Value *If_Statement::code_gen(llvm::Module *mod)
{
    return 0;
}

llvm::Value *Return_Statement::code_gen(llvm::Module *mod)
{
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
        //? This is a Sandscript function
        name = name.substr(1, name.size() - 1);
        // if (name == "print")
        // {
        // if (!print_function_declared)
        // declare_function("print", std::vector<llvm::Type *>(1, llvm::Type::getInt8PtrTy(context)));
        // }
    }
    auto callee_function = mod->getFunction(name);
    if (callee_function == 0)
        fatal_error("Unknown function referenced in function call");
    if (callee_function->arg_size() != params.size())
        fatal_error("Incorrect number of parameters passed to function call");

    std::vector<llvm::Value *> param_values;
    for (auto &param : params)
    {
        auto v = param->code_gen(mod);
        param_values.push_back(v);
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