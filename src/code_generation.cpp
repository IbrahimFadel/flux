#include "code_generation.h"

unique_ptr<Module> code_gen_nodes(const Nodes &nodes)
{
    modules[current_module_pointer] = std::make_unique<Module>("TheModule", context);

    for (auto &node : nodes)
    {
        code_gen_node(std::move(node));
    }

    print_current_module();

    return std::move(modules[current_module_pointer]);
}

Value *code_gen_node(const unique_ptr<Node> &node)
{
    node->code_gen();
    return 0;
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

    if (verifyFunction(*function, &outs()))
    {
        print_current_module();
        exit(1);
    }

    return function;
}

void Prototype_Node::create_argument_allocas(Function *function)
{
    Function::arg_iterator it = function->arg_begin();
    for (unsigned i = 0, size = param_names.size(); i != size; ++i, ++it)
    {
        auto ptr = create_entry_block_alloca(function, param_names[i], variable_type_to_llvm_type(param_types[i]));
        auto store = builder.CreateStore(it, ptr);
        auto loaded = builder.CreateLoad(ptr);
        functions[current_function_name]->set_variable_ptr(param_names[i], ptr);
        functions[current_function_name]->set_variable_value(param_names[i], loaded);
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
    for (auto &param_type : param_types)
    {
        auto llvm_type = variable_type_to_llvm_type(param_type);
        types.push_back(llvm_type);
    }

    auto ret = variable_type_to_llvm_type(return_type);

    llvm::FunctionType *function_type = llvm::FunctionType::get(ret, types, false);
    llvm::Function *f = llvm::Function::Create(function_type, llvm::Function::ExternalLinkage, name, modules[current_module_pointer].get());

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
        return code_gen_object_variable_declaration(this);
    else
        return code_gen_primitive_variable_declaration(this);
}

Value *code_gen_object_variable_declaration(Variable_Declaration_Node *var)
{
    auto llvm_ty = object_types[var->get_type_name()];
    auto ptr = builder.CreateAlloca(llvm_ty, 0, var->get_name());

    define_object_properties(ptr, var->get_value());

    auto loaded = builder.CreateLoad(ptr);

    return 0;
}

void define_object_properties(Value *ptr, unique_ptr<Node> expr)
{
    auto properties = expr->get_properties();
    auto it = properties.begin();
    for (unsigned i = 0; it != properties.end(); i++)
    {
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
    auto v = var->get_value()->code_gen();
    auto store = builder.CreateStore(v, ptr);
    auto loaded = builder.CreateLoad(ptr);

    functions[current_function_name]->set_variable_ptr(var->get_name(), ptr);
    functions[current_function_name]->set_variable_value(var->get_name(), loaded);

    return store;
}

Value *If_Node::code_gen()
{
    return 0;
}
Value *For_Node::code_gen()
{
    return 0;
}
Value *Condition_Node::code_gen()
{
    return 0;
}
Value *Function_Call_Node::code_gen()
{
    auto callee_function = modules[current_module_pointer]->getFunction(name);
    if (callee_function == 0)
        error("Unknown function referenced in function call");
    if (callee_function->arg_size() != parameters.size())
        error("Incorrect number of parameters passed to function call");

    std::vector<llvm::Value *> args_v;
    for (unsigned i = 0, size = parameters.size(); i != size; i++)
    {
        auto v = load_if_ptr(parameters[i]->code_gen());
        args_v.push_back(v);
    }

    auto call = builder.CreateCall(callee_function, args_v, "calltmp");

    return call;
}
Value *Variable_Reference_Node::code_gen()
{
    return functions[current_function_name]->get_variable_value(name);
}
Value *Import_Node::code_gen()
{
    return 0;
}
Value *Variable_Assignment_Node::code_gen()
{
    return 0;
}
Value *Object_Node::code_gen()
{
    auto it = properties.begin();

    std::vector<llvm::Type *> members;
    while (it != properties.end())
    {
        members.push_back(variable_type_to_llvm_type(it->second));
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
    return 0;
}

Value *Return_Node::code_gen()
{
    auto v = value->code_gen();
    auto ret = builder.CreateRet(v);
    return ret;
}

Type *variable_type_to_llvm_type(Variable_Type type)
{
    switch (type)
    {
    case Variable_Type::type_i64:
        return Type::getInt32Ty(context);
    case Variable_Type::type_i32:
        return Type::getInt32Ty(context);
    case Variable_Type::type_i16:
        return Type::getInt32Ty(context);
    case Variable_Type::type_i8:
        return Type::getInt32Ty(context);
    case Variable_Type::type_bool:
        return Type::getInt32Ty(context);
    case Variable_Type::type_float:
        return Type::getInt32Ty(context);
    case Variable_Type::type_double:
        return Type::getInt32Ty(context);
    default:
        error("Could not convert variable type to llvm type");
        return nullptr;
    }
}

Value *load_if_ptr(Value *v)
{
    if (v->getType()->isPointerTy())
        return builder.CreateLoad(v);
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