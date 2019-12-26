#ifndef _DOCPARSE_H
#define _DOCPARSE_H

#define WASM_EXPORT __attribute__ ((visibility("default")))

WASM_EXPORT const char* parse(const char* input, const char* path);

#endif
