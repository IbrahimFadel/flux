#ifndef OPTIONS_H
#define OPTIONS_H

#include <string>

struct CompilerOptions
{
    bool optimize = false;
    std::string path_to_root = "/home/ibrahim/dev/sandscriptold/";
    std::string output_path = path_to_root + std::string("out.ll");
    bool print_module = false;
};

#endif