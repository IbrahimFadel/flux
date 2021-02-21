#include "driver.h"

using namespace ssc;

void Driver::parseCommandLineArguments(std::vector<std::string> args)
{
    auto opts = std::make_unique<Options>();

    opts->setInputFilePath(args[1]);

    int i = 0;
    for (auto &arg : args)
    {
        if (arg == "--optimize")
            opts->setOptimize(true);
        else if (arg == "--output" || arg == "-o")
            opts->setOutputFilePath(std::string(args[i + 1]));
        else if (arg == "--print" || arg == "-p")
            opts->setDebug(true);
        i++;
    }

    options = std::move(opts);
}

void Driver::error(std::string msg)
{
    std::cerr << "\033[1;31m"
              << "Driver Error: "
              << "\033[0m" << msg << std::endl;
    exit(1);
}

void Driver::warning(std::string msg)
{
    std::cerr << "\033[1;33m"
              << "Warning: "
              << "\033[0m" << msg << std::endl;
}

void Driver::info(std::string msg)
{
    std::cerr << "\033[1;37m"
              << "Info: "
              << "\033[0m" << msg << std::endl;
}

Options *Driver::getOptions()
{
    return options.get();
}

std::vector<std::string> Driver::getFileContent(const char *path)
{
    std::vector<std::string> content;
    std::ifstream input(path);
    for (std::string line; getline(input, line);)
    {
        line += '\n';
        content.push_back(line);
    }
    return content;
}

void Driver::compile(std::string strPath)
{
    std::error_code ec;
    auto path = fs::canonical(strPath, ec);
    if (ec)
    {
        error("Could not find file: " + strPath);
    }

    std::vector<std::string> fileContent = getFileContent(path.c_str());

    auto lexer = std::make_unique<Lexer>();
    auto tokens = lexer->tokenize(fileContent); //? might need to reset col/row

    if (options->getDebug())
        lexer->printTokens(tokens);

    auto parser = std::make_unique<Parser>();
    auto nodes = parser->parseTokens(std::move(tokens));

    auto codeGenerator = std::make_unique<CodeGenerator>();
    codeGenerator->codegenNodes(std::move(nodes));
}