#include "ssc.h"
#include "driver/driver.h"

using std::cout;
using std::endl;

int main(int argc, const char **argv)
{
    std::vector<std::string> arguments(argv, argv + argc);
    auto driver = std::make_unique<ssc::Driver>();
    driver->parseCommandLineArguments(arguments);

    auto options = driver->getOptions();
    auto path = options->getInputFilePath();
    driver->compile(path);

    return 0;
}