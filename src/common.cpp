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

void error(const char *arg, int line, int column)
{
    if (line == UNKOWN_LINE || column == UNKNOWN_COLUMN)
        std::cout << arg << std::endl;
    else
        std::cout << arg << " at line " << line << " column " << column << std::endl;

    exit(1);
}