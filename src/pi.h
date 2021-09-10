#ifndef PI_H
#define PI_H

#include <cvec.h>

struct Package;

#include "ast.h"

typedef struct Package {
  const char *name;
  cvector_vector_type(FnDecl) public_functions;
  cvector_vector_type(FnDecl) private_functions;
  cvector_vector_type(VarDecl) public_variables;
  cvector_vector_type(VarDecl) private_variables;
  cvector_vector_type(TypeDecl) public_types;
  cvector_vector_type(TypeDecl) private_types;
} Package;

Package *package_create();
void package_print(Package *p);

#endif