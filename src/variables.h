#ifndef VARIABLES_H
#define VARIABLES_H

#include <iostream>

namespace Variables
{

typedef enum Variable_Types
{
  integer
} Variable_Types;

typedef struct Variable
{
  int type;
  std::string name;
  int intValue;
} Variable;
} // namespace Variables

#endif