#ifndef COMMON_H
#define COMMON_H

#include <fstream>
#include <string>
#include <vector>
#include <iostream>
#include <cstdarg>

#include <llvm/IR/Module.h>

using std::unique_ptr;

std::vector<std::string> get_file_content(const char *path);

class Program
{
private:
    std::string name;
    unique_ptr<llvm::Module> module;
    std::vector<unique_ptr<Program>> imported_programs;

public:
    Program(std::string name) : name(name){};
    unique_ptr<llvm::Module> get_module();
};

#endif