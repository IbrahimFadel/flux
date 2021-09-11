#include "pi.h"

#include <cvec.h>
#include <stdio.h>
#include <stdlib.h>

#include "ir.h"
#include "parser.h"
#include "scanner.h"

Package *package_create() {
  Package *pkg = malloc(sizeof(Package));
  pkg->name = NULL;
  pkg->private_functions = NULL;
  pkg->public_functions = NULL;
  pkg->private_variables = NULL;
  pkg->public_variables = NULL;
  pkg->private_types = NULL;
  pkg->public_types = NULL;
  return pkg;
}

void package_print(Package *p) {
  printf("+----- PKG: %s -----+\n", p->name);
  printf("| Public Functions:\n");
  FnDecl *it;
  for (it = cvector_begin(p->public_functions); it != cvector_end(p->public_functions); it++) {
    printf("| \t%s\n", it->name);
  }
  printf("| Private Functions:\n");
  for (it = cvector_begin(p->private_functions); it != cvector_end(p->private_functions); it++) {
    printf("| \t%s\n", it->name);
  }
}

int main() {
  const char *file_content = read_file("./test.pi");
  if (file_content == NULL) {
    printf("Could not read file content\n");
    exit(1);
  }
  printf("==== File Content ====\n%s\n======================\n", file_content);

  Scanner *s = scanner_create(file_content);
  Token *tokens = scan_file(s);

  int i = 0;
  while (tokens[i].type != TOKTYPE_EOF) {
    printf("%s:%d\n", tokens[i].value, tokens[i].type);
    i++;
  }

  ParseContext *ctx = parsecontext_create(tokens);
  parse_pkg_file_tokens(ctx);

  Package *pkg = package_create();
  pkg->name = ctx->pkg;
  FnDecl *it;
  for (it = cvector_begin(ctx->functions); it != cvector_end(ctx->functions); it++) {
    if (it->pub) {
      cvector_push_back(pkg->public_functions, *it);
    } else {
      cvector_push_back(pkg->private_functions, *it);
    }
  }

  package_print(pkg);

  // LLVMModuleRef mod = codegen_pkg(pkg);
  // printf("======= Module =======\n");
  // LLVMDumpModule(mod);
  // printf("======================\n");

  free(s);
  free(tokens);
  free(ctx);
  free(pkg);

  return 0;
}