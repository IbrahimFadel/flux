#include "pi.h"

#include <cvec.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "ir.h"
#include "parser.h"
#include "scanner.h"
#include "typecheck.h"

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
  FnDecl *f_it;
  for (f_it = cvector_begin(p->public_functions); f_it != cvector_end(p->public_functions); f_it++) {
    printf("| \t%s\n", f_it->name);
  }
  printf("| Private Functions:\n");
  for (f_it = cvector_begin(p->private_functions); f_it != cvector_end(p->private_functions); f_it++) {
    printf("| \t%s\n", f_it->name);
  }
  printf("| Public Types:\n");
  TypeDecl *t_it;
  for (t_it = cvector_begin(p->public_types); t_it != cvector_end(p->public_types); t_it++) {
    printf("| \t%s\n", t_it->name);
  }
  printf("| Private Types:\n");
  for (t_it = cvector_begin(p->private_types); t_it != cvector_end(p->private_types); t_it++) {
    printf("| \t%s\n", t_it->name);
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
    FnDecl *f_it;
    for (f_it = cvector_begin(ctx->functions); f_it != cvector_end(ctx->functions); f_it++) {
      if (f_it->pub) {
        cvector_push_back(pkg_it->public_functions, *f_it);
      } else {
        cvector_push_back(pkg_it->private_functions, *f_it);
      }
    }
    TypeDecl *t_it;
    for (t_it = cvector_begin(ctx->types); t_it != cvector_end(ctx->types); t_it++) {
      if (t_it->pub) {
        cvector_push_back(pkg_it->public_types, *t_it);
      } else {
        cvector_push_back(pkg_it->private_types, *t_it);
      }
    }

    free(s);
    free(tokens);
    free(ctx);
  }

  Package *pkg_it = NULL;
  TypecheckContext *typecheck_ctx = typecheck_ctx_create(pkg_it);
  for (pkg_it = cvector_begin(packages); pkg_it != cvector_end(packages); pkg_it++) {
    typecheck_pkg(typecheck_ctx, pkg_it);
    package_print(pkg_it);
    LLVMModuleRef mod = codegen_pkg(typecheck_ctx);
    printf("===== IR Module =====\n");
    LLVMDumpModule(mod);
    printf("=====================\n");

    char *err_msg[2] = {"could not write module to file", NULL};
    char *file_ext = ".ll";
    char *file_name = malloc(strlen(pkg_it->name) + strlen(file_ext));
    strcpy(file_name, pkg_it->name);
    strcat(file_name, file_ext);

    LLVMPrintModuleToFile(mod, file_name, err_msg);
  }
  free(typecheck_ctx);

  return 0;
}