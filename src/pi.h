#ifndef PI_H
#define PI_H

#include <c-vector/cvector.h>
#include <sds/sds.h>

struct Package;

#include "ast.h"
#include "parser.h"

typedef struct Package {
  const char *name;
  cvector_vector_type(FnDecl *) public_functions;
  cvector_vector_type(FnDecl *) private_functions;
  cvector_vector_type(VarDecl *) public_variables;
  cvector_vector_type(VarDecl *) private_variables;
  cvector_vector_type(TypeDecl *) public_types;
  cvector_vector_type(TypeDecl *) private_types;
} Package;

Package *package_create();
void package_destroy(Package *pkg);
sds package_tostring(Package *pkg);

cvector_vector_type(const char *) get_input_files(int argc, char **argv);
Package *insert_package(Package *pkgs, struct ParseContext *ctx);
void add_declarations_to_package(Package *pkg, cvector_vector_type(FnDecl *) cstd_functions, struct ParseContext *ctx);

#endif