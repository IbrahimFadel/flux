#ifndef SSC_DRIVER_DRIVER_H
#define SSC_DRIVER_DRIVER_H

#include <memory>
#include <vector>
#include <string>
#include <iostream>
#include <filesystem>
#include <fstream>
#include <map>

#include "options.h"
#include "ast/lexer.h"
#include "ast/parser.h"
#include "dependencies.h"
#include "ir/context.h"
#include "ir/codegen.h"
#include "linker/lowering.h"

using std::unique_ptr;
namespace fs = std::filesystem;

namespace ssc
{
    class Driver
    {
    private:
        unique_ptr<Options> options;
        std::map<fs::path, Nodes> fileASTs;

        void error(std::string msg);
        void warning(std::string msg);
        void info(std::string msg);

        std::vector<std::string> getFileContent(const char *path);
        void writeLLFile(const unique_ptr<CodegenContext> &codegenContext, std::string path);
        unique_ptr<DependencyGraph> createDependencyGraph(fs::path basePath, const Nodes &nodes);
        void addDependencyConnections(const unique_ptr<DependencyGraph> &graph, fs::path parentPath, const Nodes &nodes);

    public:
        void parseCommandLineArguments(std::vector<std::string> args);
        void compile(std::vector<std::string> path);

        Options *getOptions();
    };
} // namespace ssc

#endif