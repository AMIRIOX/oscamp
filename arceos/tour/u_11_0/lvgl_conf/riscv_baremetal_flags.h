#ifndef RISCV_BAREMETAL_FLAGS_H
#define RISCV_BAREMETAL_FLAGS_H

// This file is a workaround for bare-metal C cross-compilation issues.
// It provides minimal definitions for types that would otherwise be in
// headers like <stdint.h> and <stddef.h>, which are not available in a
// `-nostdinc` environment.

// From <stddef.h>
#ifndef __SIZE_TYPE__
#define __SIZE_TYPE__ long unsigned int
#endif
#if !defined(NULL)
#define NULL ((void*)0)
#endif
typedef __SIZE_TYPE__ size_t;

// Basic integer types from <stdint.h>
// These rely on the compiler's built-in knowledge of types for the target.
typedef __UINT8_TYPE__   uint8_t;
typedef __UINT16_TYPE__  uint16_t;
typedef __UINT32_TYPE__  uint32_t;
typedef __UINT64_TYPE__  uint64_t;
typedef __INT8_TYPE__    int8_t;
typedef __INT16_TYPE__   int16_t;
typedef __INT32_TYPE__   int32_t;
typedef __INT64_TYPE__   int64_t;
typedef __UINTPTR_TYPE__ uintptr_t;

#endif // RISCV_BAREMETAL_FLAGS_H
