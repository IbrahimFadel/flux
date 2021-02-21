#include "options.h"

using namespace ssc;

void Options::setOptimize(bool v)
{
    optimize = v;
}

void Options::setOutputFilePath(std::string path)
{
    outputFilePath = path;
}

void Options::setInputFilePath(std::string path)
{
    inputFilePath = path;
}

void Options::setDebug(bool v)
{
    debug = v;
}

std::string Options::getInputFilePath()
{
    return inputFilePath;
}

bool Options::getDebug()
{
    return debug;
}

bool Options::getOptimize()
{
    return optimize;
};