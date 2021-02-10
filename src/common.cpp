#include "common.h"

std::vector<std::string> get_file_content(const char *path)
{
    std::vector<std::string> content;
    std::ifstream input(path);
    for (std::string line; getline(input, line);)
    {
        content.push_back(line);
    }
    return content;
}

// unique_ptr<llvm::Module> Program::get_module()
// {
//     return std::move(module);
// }