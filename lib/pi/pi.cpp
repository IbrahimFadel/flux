#include "pi.h"
#include "driver/driver.h"

using std::cout;
using std::endl;

int main(int argc, const char **argv)
{
    std::vector<std::string> arguments(argv, argv + argc);
    arguments.erase(arguments.begin());
    auto driver = std::make_unique<ssc::Driver>();
    driver->parseCommandLineArguments(arguments);

    auto options = driver->getOptions();
    auto paths = options->getInputFilePaths();
    driver->compile(paths);

    return 0;
}