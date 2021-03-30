#ifndef SSC_DRIVER_OPTIONS_H
#define SSC_DRIVER_OPTIONS_H

#include <vector>
#include <string>
#include <filesystem>
#include <map>

namespace ssc
{
    enum OutputType
    {
        object,
        assembly,
        llvmIr
    };

    enum CodegenWarnings
    {
        UnnecessaryTypecast
    };

    class Options
    {
    private:
        bool optimize;
        std::string outputFilePath;
        OutputType outputType = OutputType::llvmIr;
        bool debug;
        std::vector<std::string> inputFilePaths;
        std::map<CodegenWarnings, bool> codegenWarnings = {
            {UnnecessaryTypecast, true}};
        bool wError = false;

    public:
        void enableAllCodegenWarnings();
        void enableWarning(CodegenWarnings warning) { codegenWarnings[warning] = true; }
        void disableWarning(CodegenWarnings warning) { codegenWarnings[warning] = false; }
        void setWError(bool v) { wError = v; }

        void setOptimize(bool v) { optimize = v; }
        void setOutputFilePath(std::string path) { outputFilePath = path; }
        void setInputFilePaths(std::vector<std::string> paths) { inputFilePaths = paths; }
        void setDebug(bool v) { debug = v; }

        std::vector<std::string> getInputFilePaths() { return inputFilePaths; }
        std::string getOutputFilePath() { return outputFilePath; }
        bool getDebug() { return debug; }
        bool getOptimize() { return optimize; }
        std::map<CodegenWarnings, bool> getCodegenWarnings() { return codegenWarnings; }
        bool getWError() { return wError; }
    };
} // namespace ssc

#endif