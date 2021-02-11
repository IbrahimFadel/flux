#ifndef OPTIONS_H
#define OPTIONS_H

#include <filesystem>
#include <string>

struct CompilerOptions
{
    bool optimize = false;
    std::string path_to_root = std::filesystem::current_path().string();
    std::string output_path = path_to_root + std::string("a.out");
    bool print_module = false;
};

#endif