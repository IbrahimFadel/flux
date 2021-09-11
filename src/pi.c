#include "pi.h"

#include <cvec.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

int main(int argc, char **argv) {
  cvector_vector_type(const char *) input_files = NULL;
  if (argc == 1) {
    printf("Specify input files\n");
    exit(1);
  } else {
    int i;
    for (i = 0; i < argc - 1; i++) {
      cvector_push_back(input_files, argv[i + 1]);
    }
  }

  cvector_vector_type(Package) packages = NULL;

  int i;
  for (i = 0; i < cvector_size(input_files); i++) {
    const char *file_content = read_file(input_files[i]);
    if (file_content == NULL) {
      printf("Could not read file content\n");
      exit(1);
    }

    Scanner *s = scanner_create(file_content);
    Token *tokens = scan_file(s);

    ParseContext *ctx = parsecontext_create(tokens);
    parse_pkg_file_tokens(ctx);

    Package *pkg_it;
    bool pkg_found = false;
    for (pkg_it = cvector_begin(packages); pkg_it != cvector_end(packages); pkg_it++) {
      if (!strcmp(pkg_it->name, ctx->pkg)) {
        pkg_found = true;
        break;
      }
    }
    if (!pkg_found) {
      cvector_push_back(packages, *package_create());
      pkg_it = &packages[cvector_size(packages) - 1];
      pkg_it->name = ctx->pkg;
    }
    FnDecl *it;
    for (it = cvector_begin(ctx->functions); it != cvector_end(ctx->functions); it++) {
      if (it->pub) {
        cvector_push_back(pkg_it->public_functions, *it);
      } else {
        cvector_push_back(pkg_it->private_functions, *it);
      }
    }

    free(s);
    free(tokens);
    free(ctx);
  }

  Package *pkg_it;
  for (pkg_it = cvector_begin(packages); pkg_it != cvector_end(packages); pkg_it++) {
    package_print(pkg_it);
    LLVMModuleRef mod = codegen_pkg(pkg_it);
    printf("===== IR Module =====\n");
    LLVMDumpModule(mod);
    printf("=====================\n");
  }

  return 0;
}