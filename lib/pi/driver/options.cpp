#include "options.h"

using namespace ssc;

void Options::enableAllCodegenWarnings()
{
    for (auto &[key, val] : codegenWarnings)
    {
        val = true;
    }
}