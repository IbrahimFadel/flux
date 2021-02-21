#ifndef SSC_DRIVER_OPTIONS_H
#define SSC_DRIVER_OPTIONS_H

#include <string>
#include <filesystem>

namespace ssc
{
    enum OutputType
    {
        object,
        assembly,
        llvmIr
    };

    class Options
    {
    private:
        bool optimize;
        std::string outputFilePath;
        OutputType outputType = OutputType::llvmIr;
        bool debug;
        std::string inputFilePath;

    public:
        void setOptimize(bool v);
        void setOutputFilePath(std::string path);
        void setInputFilePath(std::string path);
        void setDebug(bool v);

        std::string getInputFilePath();
        bool getDebug();
        bool getOptimize();
    };
} // namespace ssc

#endif