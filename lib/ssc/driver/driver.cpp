#include "driver.h"

using namespace ssc;

void Driver::parseCommandLineArguments(std::vector<std::string> args)
{
    auto opts = std::make_unique<Options>();

    std::vector<std::string> inputFilePaths;

    for (int i = 0; i < args.size(); i++)
    {
        auto arg = args[i];
        if (arg == "--optimize")
        {
            opts->setOptimize(true);
        }
        else if (arg == "--output" || arg == "-o")
        {
            opts->setOutputFilePath(std::string(args[i + 1]));
            i++;
        }
        else if (arg == "--print" || arg == "-p")
        {
            opts->setDebug(true);
        }
        else if (arg == "--warn-all")
        {
            opts->enableAllCodegenWarnings();
        }
        else if (arg == "-Wunnecessary-typecast")
        {
            opts->enableWarning(CodegenWarnings::UnnecessaryTypecast);
        }
        else if (arg == "-Wno-unnecessary-typecast")
        {
            opts->disableWarning(CodegenWarnings::UnnecessaryTypecast);
        }
        else if (arg == "-Werror")
        {
            opts->setWError(true);
        }
        else
        {
            inputFilePaths.push_back(arg);
        }
    }

    opts->setInputFilePaths(inputFilePaths);

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

void Driver::compile(std::vector<std::string> paths)
{
    std::string linkingCMD = "clang -g ";
    std::vector<std::string> objOutPaths;
    for (auto &path : paths)
    {
        std::error_code ec;
        auto fsInputPath = fs::canonical(path, ec);
        if (ec)
        {
            error("Could not find file: " + path);
        }

        std::vector<std::string> fileContent = getFileContent(fsInputPath.c_str());

        auto lexer = std::make_unique<Lexer>();
        auto tokens = lexer->tokenize(fileContent);

        auto parser = std::make_unique<Parser>();
        auto astNodes = parser->parseTokens(std::move(tokens));

        auto codegenCtx = std::make_unique<CodegenContext>(fsInputPath.string(), options);
        // codegenCtx->init(fsInputPath.string());
        codegenNodes(std::move(astNodes), codegenCtx);

        auto objOutPath = fsInputPath.replace_extension("o");
        objOutPaths.push_back(objOutPath.string());
        writeModuleToObjectFile(codegenCtx, objOutPath.string());
        if (options->getDebug())
            codegenCtx->printModule();

        linkingCMD += objOutPath.string() + " ";
    }

    linkingCMD += "-o " + options->getOutputFilePath();

    int exitCode;
    info("Invoking command: " + linkingCMD);
    executeCommand(linkingCMD, exitCode);
    info("Linker exit code: " + std::to_string(exitCode));
}