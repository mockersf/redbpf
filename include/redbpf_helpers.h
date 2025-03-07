#include <linux/kconfig.h>
#include <linux/types.h>
#ifdef asm_volatile_goto
#undef asm_volatile_goto
#define asm_volatile_goto(x...) asm volatile("invalid use of asm_volatile_goto")
#endif
#include <linux/version.h>
#include <uapi/linux/ptrace.h>
#include <linux/bpf.h>
#include "bpf_helpers.h"
#include "xdp.h"
