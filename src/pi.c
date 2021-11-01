#include "pi.h"

#include <c-vector/cvector.h>
#include <sds/sds.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "error.h"
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

void package_destroy(Package *pkg) {
  unsigned i;
  for (i = 0; i < cvector_size(pkg->private_functions); i++) {
    fndecl_destroy(pkg->private_functions[i]);
  }
  cvector_free(pkg->private_functions);
  for (i = 0; i < cvector_size(pkg->public_functions); i++) {
    fndecl_destroy(pkg->public_functions[i]);
  }
  cvector_free(pkg->public_functions);
  for (i = 0; i < cvector_size(pkg->private_types); i++) {
    typedecl_destroy(pkg->private_types[i]);
  }
  cvector_free(pkg->private_types);
  for (i = 0; i < cvector_size(pkg->public_types); i++) {
    typedecl_destroy(pkg->public_types[i]);
  }
  cvector_free(pkg->public_types);
  // cvector_free(pkg->private_variables);
  // cvector_free(pkg->public_variables);
}

void package_print(Package *p) {
  printf("+----- PKG: %s -----+\n", p->name);
  printf("| Public Functions:\n");
  FnDecl **f_it;
  for (f_it = cvector_begin(p->public_functions); f_it != cvector_end(p->public_functions); f_it++) {
    printf("| \t%s\n", (*f_it)->name);
  }
  printf("| Private Functions:\n");
  for (f_it = cvector_begin(p->private_functions); f_it != cvector_end(p->private_functions); f_it++) {
    printf("| \t%s\n", (*f_it)->name);
  }
  printf("| Public Types:\n");
  TypeDecl **t_it;
  for (t_it = cvector_begin(p->public_types); t_it != cvector_end(p->public_types); t_it++) {
    printf("| \t%s\n", (*t_it)->name);
  }
  printf("| Private Types:\n");
  for (t_it = cvector_begin(p->private_types); t_it != cvector_end(p->private_types); t_it++) {
    printf("| \t%s\n", (*t_it)->name);
  }
}

sds package_tostring(Package *p) {
  sds repr = sdsnew("+----- PKG -----+\n");
  printf("| Public Functions:\n");
  FnDecl **f_it;
  for (f_it = cvector_begin(p->public_functions); f_it != cvector_end(p->public_functions); f_it++) {
    repr = sdscat(repr, fn_tostring(*f_it));
  }
  printf("| Private Functions:\n");
  for (f_it = cvector_begin(p->private_functions); f_it != cvector_end(p->private_functions); f_it++) {
    repr = sdscat(repr, fn_tostring(*f_it));
  }
  printf("| Public Types:\n");
  TypeDecl **t_it;
  for (t_it = cvector_begin(p->public_types); t_it != cvector_end(p->public_types); t_it++) {
  }
  printf("| Private Types:\n");
  for (t_it = cvector_begin(p->private_types); t_it != cvector_end(p->private_types); t_it++) {
  }
  return repr;
}

cvector_vector_type(FnDecl *) create_cstd_functions() {
  cvector_vector_type(FnDecl *) functions = NULL;
  const char *file_content = "fn malloc(u64 size) -> i8*;\n";
  Scanner *s = scanner_create();
  s->src = file_content;
  s->ch = s->src[s->offset];
  cvector_vector_type(Token *) tokens = scan_file(s);
  ParseContext *ctx = parsecontext_create(tokens);
  FnDecl *malloc_decl = parse_fn_decl(ctx, false);
  cvector_push_back(functions, malloc_decl);

  file_content = "fn free(i8 *buf);\n";
  s->src = file_content;
  s->offset = 0;
  s->ch = file_content[0];
  tokens = scan_file(s);
  ctx->toks = tokens;
  ctx->tok_ptr = 0;
  ctx->cur_tok = tokens[0];
  FnDecl *free_decl = parse_fn_decl(ctx, false);
  cvector_push_back(functions, free_decl);

  file_content = "fn memcpy(i8 *buf1, i8 *buf2, u64 size);\n";
  s->src = file_content;
  s->offset = 0;
  s->ch = file_content[0];
  tokens = scan_file(s);
  ctx->toks = tokens;
  ctx->tok_ptr = 0;
  ctx->cur_tok = tokens[0];
  FnDecl *memcpy_decl = parse_fn_decl(ctx, false);
  cvector_push_back(functions, memcpy_decl);

  scanner_destroy(s);
  parsecontext_destroy(ctx);
  return functions;
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
  cvector_vector_type(cvector_vector_type(Token *)) tokens_list = NULL;
  cvector_vector_type(FnDecl *) cstd_functions = create_cstd_functions();

  Scanner *s = scanner_create();
  int i;
  for (i = 0; i < cvector_size(input_files); i++) {
    const char *file_content = read_file(input_files[i]);
    if (file_content == NULL) {
      log_error(ERRTYPE_DRIVER, "could not read file content");
    }

    scanner_reset(s);
    s->src = file_content;
    s->ch = s->src[s->offset];
    cvector_vector_type(Token *) tokens = scan_file(s);
    cvector_push_back(tokens_list, tokens);

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
    FnDecl **f_it;
    for (f_it = cvector_begin(cstd_functions); f_it != cvector_end(cstd_functions); f_it++) {
      cvector_push_back(pkg_it->private_functions, *f_it);
    }
    for (f_it = cvector_begin(ctx->functions); f_it != cvector_end(ctx->functions); f_it++) {
      if ((*f_it)->pub) {
        cvector_push_back(pkg_it->public_functions, *f_it);
      } else {
        cvector_push_back(pkg_it->private_functions, *f_it);
      }
    }
    TypeDecl **t_it;
    for (t_it = cvector_begin(ctx->types); t_it != cvector_end(ctx->types); t_it++) {
      if ((*t_it)->pub) {
        cvector_push_back(pkg_it->public_types, *t_it);
      } else {
        cvector_push_back(pkg_it->private_types, *t_it);
      }
    }
    VarDecl **v_it;
    for (v_it = cvector_begin(ctx->variables); v_it != cvector_end(ctx->variables); v_it++) {
      if ((*v_it)->pub) {
        cvector_push_back(pkg_it->public_variables, *v_it);
      } else {
        cvector_push_back(pkg_it->private_variables, *v_it);
      }
    }

    scanner_destroy(s);
    parsecontext_destroy(ctx);
  }
  cvector_free(input_files);

  Package *pkg_it = NULL;
  TypecheckContext *typecheck_ctx = typecheck_ctx_create(pkg_it);
  for (pkg_it = cvector_begin(packages); pkg_it != cvector_end(packages); pkg_it++) {
    typecheck_pkg(typecheck_ctx, pkg_it);
    // package_print(pkg_it);
    sds str = package_tostring(pkg_it);
    // printf("%s\n", str);
    sdsfree(str);
    LLVMModuleRef mod = codegen_pkg(typecheck_ctx);
    printf("===== IR Module =====\n");
    LLVMDumpModule(mod);
    printf("=====================\n");

    char *err_msg[2] = {"could not write module to file", NULL};
    char *file_ext = ".ll";
    char *file_name = malloc(strlen(pkg_it->name) + strlen(file_ext) + 1);  // add 1 for '\0'
    strcpy(file_name, pkg_it->name);
    strcat(file_name, file_ext);

    LLVMPrintModuleToFile(mod, file_name, err_msg);

    LLVMDisposeModule(mod);
  }

  // typecheck_ctx_destroy(typecheck_ctx);

  // for (i = 0; i < cvector_size(tokens_list); i++) {
  //   unsigned j;
  //   for (j = 0; j < cvector_size(tokens_list[i]); j++) {
  //     token_destroy(tokens_list[i][j]);
  //   }
  //   cvector_free(tokens_list[i]);
  // }
  // cvector_free(tokens_list);

  // cvector_free(packages);

  return 0;
}

// #include "pi.h"

// #include <c-vector/cvector.h>
// #include <sds/sds.h>
// #include <stdio.h>
// #include <stdlib.h>
// #include <string.h>

// #include "error.h"
// #include "ir.h"
// #include "parser.h"
// #include "scanner.h"
// #include "typecheck.h"

// Package *package_create() {
//   Package *pkg = malloc(sizeof(Package));
//   pkg->name = NULL;
//   pkg->private_functions = NULL;
//   pkg->public_functions = NULL;
//   pkg->private_variables = NULL;
//   pkg->public_variables = NULL;
//   pkg->private_types = NULL;
//   pkg->public_types = NULL;
//   return pkg;
// }

// void package_destroy(Package *pkg) {
//   unsigned i;
//   for (i = 0; i < cvector_size(pkg->private_functions); i++) {
//     fndecl_destroy(pkg->private_functions[i]);
//   }
//   cvector_free(pkg->private_functions);
//   for (i = 0; i < cvector_size(pkg->public_functions); i++) {
//     fndecl_destroy(pkg->public_functions[i]);
//   }
//   cvector_free(pkg->public_functions);
//   for (i = 0; i < cvector_size(pkg->private_types); i++) {
//     typedecl_destroy(pkg->private_types[i]);
//   }
//   cvector_free(pkg->private_types);
//   for (i = 0; i < cvector_size(pkg->public_types); i++) {
//     typedecl_destroy(pkg->public_types[i]);
//   }
//   cvector_free(pkg->public_types);
//   // cvector_free(pkg->private_variables);
//   // cvector_free(pkg->public_variables);
// }

// sds package_tostring(Package *p) {
//   sds repr = sdsnew("+----- PKG -----+\n");
//   printf("| Public Functions:\n");
//   FnDecl **f_it;
//   for (f_it = cvector_begin(p->public_functions); f_it != cvector_end(p->public_functions); f_it++) {
//     repr = sdscat(repr, fn_tostring(*f_it));
//   }
//   printf("| Private Functions:\n");
//   for (f_it = cvector_begin(p->private_functions); f_it != cvector_end(p->private_functions); f_it++) {
//     repr = sdscat(repr, fn_tostring(*f_it));
//   }
//   printf("| Public Types:\n");
//   TypeDecl **t_it;
//   for (t_it = cvector_begin(p->public_types); t_it != cvector_end(p->public_types); t_it++) {
//   }
//   printf("| Private Types:\n");
//   for (t_it = cvector_begin(p->private_types); t_it != cvector_end(p->private_types); t_it++) {
//   }
//   return repr;
// }

// cvector_vector_type(FnDecl *) create_cstd_functions() {
//   cvector_vector_type(FnDecl *) functions = NULL;
//   const char *file_content = "fn malloc(u64 size) -> i8*;\n";
//   Scanner *s = scanner_create();
//   s->src = file_content;
//   s->ch = s->src[s->offset];
//   cvector_vector_type(Token *) tokens = scan_file(s);
//   ParseContext *ctx = parsecontext_create(tokens);
//   FnDecl *malloc_decl = parse_fn_decl(ctx, false);
//   cvector_push_back(functions, malloc_decl);

//   file_content = "fn free(i8 *buf);\n";
//   s->src = file_content;
//   s->offset = 0;
//   s->ch = file_content[0];
//   tokens = scan_file(s);
//   ctx->toks = tokens;
//   ctx->tok_ptr = 0;
//   ctx->cur_tok = tokens[0];
//   FnDecl *free_decl = parse_fn_decl(ctx, false);
//   cvector_push_back(functions, free_decl);

//   file_content = "fn memcpy(i8 *buf1, i8 *buf2, u64 size);\n";
//   s->src = file_content;
//   s->offset = 0;
//   s->ch = file_content[0];
//   tokens = scan_file(s);
//   ctx->toks = tokens;
//   ctx->tok_ptr = 0;
//   ctx->cur_tok = tokens[0];
//   FnDecl *memcpy_decl = parse_fn_decl(ctx, false);
//   cvector_push_back(functions, memcpy_decl);

//   scanner_destroy(s);
//   parsecontext_destroy(ctx);
//   return functions;
// }

// cvector_vector_type(const char *) get_input_files(int argc, char **argv) {
//   cvector_vector_type(const char *) input_files = NULL;
//   if (argc == 1) {
//     printf("Specify input files\n");
//     exit(1);
//   } else {
//     int i;
//     for (i = 0; i < argc - 1; i++) {
//       cvector_push_back(input_files, argv[i + 1]);
//     }
//   }
//   return input_files;
// }

// Package *insert_package(Package *pkgs, ParseContext *ctx) {
//   Package *pkg_it;
//   bool pkg_found = false;
//   for (pkg_it = cvector_begin(pkgs); pkg_it != cvector_end(pkgs); pkg_it++) {
//     if (!strcmp(pkg_it->name, ctx->pkg)) {
//       pkg_found = true;
//       break;
//     }
//   }
//   if (!pkg_found) {
//     pkg_it = package_create();
//     cvector_push_back(pkgs, *pkg_it);
//     pkg_it->name = ctx->pkg;
//   }
//   return pkg_it;
// }

// void add_declarations_to_package(Package *pkg, cvector_vector_type(FnDecl *) cstd_functions, ParseContext *ctx) {
//   FnDecl **f_it;
//   for (f_it = cvector_begin(cstd_functions); f_it != cvector_end(cstd_functions); f_it++) {
//     cvector_push_back(pkg->private_functions, *f_it);
//   }
//   for (f_it = cvector_begin(ctx->functions); f_it != cvector_end(ctx->functions); f_it++) {
//     if ((*f_it)->pub) {
//       cvector_push_back(pkg->public_functions, *f_it);
//     } else {
//       cvector_push_back(pkg->private_functions, *f_it);
//     }
//   }
//   TypeDecl **t_it;
//   for (t_it = cvector_begin(ctx->types); t_it != cvector_end(ctx->types); t_it++) {
//     if ((*t_it)->pub) {
//       cvector_push_back(pkg->public_types, *t_it);
//     } else {
//       cvector_push_back(pkg->private_types, *t_it);
//     }
//   }
//   VarDecl **v_it;
//   for (v_it = cvector_begin(ctx->variables); v_it != cvector_end(ctx->variables); v_it++) {
//     if ((*v_it)->pub) {
//       cvector_push_back(pkg->public_variables, *v_it);
//     } else {
//       cvector_push_back(pkg->private_variables, *v_it);
//     }
//   }
// }

// int main(int argc, char **argv) {
//   cvector_vector_type(const char *) input_files = get_input_files(argc, argv);
//   cvector_vector_type(FnDecl *) cstd_functions = create_cstd_functions();
//   cvector_vector_type(Package) packages = NULL;

//   Scanner *s = scanner_create();
//   unsigned i;
//   for (i = 0; i < cvector_size(input_files); i++) {
//     const char *file_content = read_file(input_files[i]);
//     if (file_content == NULL) {
//       log_error(ERRTYPE_DRIVER, "could not read file content\n");
//     }

//     scanner_reset(s);
//     s->src = file_content;
//     s->ch = s->src[s->offset];

//     cvector_vector_type(Token *) tokens = scan_file(s);
//     ParseContext *ctx = parsecontext_create(tokens);
//     parse_pkg_file_tokens(ctx);

//     Package *pkg = insert_package(packages, ctx);
//     add_declarations_to_package(pkg, cstd_functions, ctx);

//     printf("hello %lu\n", cvector_size(packages));

//     parsecontext_destroy(ctx);
//   }
//   scanner_destroy(s);
//   cvector_free(input_files);

//   Package *pkg_it = NULL;
//   TypecheckContext *typecheck_ctx = typecheck_ctx_create(pkg_it);
//   for (pkg_it = cvector_begin(packages); pkg_it != cvector_end(packages); pkg_it++) {
//     typecheck_pkg(typecheck_ctx, pkg_it);
//     // sds str = package_tostring(pkg_it);
//     //   // printf("%s\n", str);
//     //   sdsfree(str);
//     LLVMModuleRef mod = codegen_pkg(typecheck_ctx);
//     printf("===== IR Module =====\n");
//     LLVMDumpModule(mod);
//     printf("=====================\n");

//     char *err_msg[2] = {"could not write module to file", NULL};
//     char *file_ext = ".ll";
//     char *file_name = malloc(strlen(pkg_it->name) + strlen(file_ext) + 1);  // add 1 for '\0'
//     strcpy(file_name, pkg_it->name);
//     strcat(file_name, file_ext);

//     LLVMPrintModuleToFile(mod, file_name, err_msg);
//     LLVMDisposeModule(mod);
//   }

//   // typecheck_ctx_destroy(typecheck_ctx);

//   // for (i = 0; i < cvector_size(tokens_list); i++) {
//   //   unsigned j;
//   //   for (j = 0; j < cvector_size(tokens_list[i]); j++) {
//   //     token_destroy(tokens_list[i][j]);
//   //   }
//   //   cvector_free(tokens_list[i]);
//   // }
//   // cvector_free(tokens_list);

//   // cvector_free(packages);

//   return 0;
// }