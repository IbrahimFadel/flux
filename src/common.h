#ifndef COMMON_H
#define COMMON_H

#include <fstream>
#include <string>
#include <vector>
#include <iostream>
#include <cstdarg>

#define UNKOWN_LINE -1
#define UNKNOWN_COLUMN -1

std::vector<std::string> get_file_content(const char *path);
void error(const char *arg, int line, int column);

#endif