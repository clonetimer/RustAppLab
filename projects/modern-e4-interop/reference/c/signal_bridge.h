#ifndef E4_SIGNAL_BRIDGE_H
#define E4_SIGNAL_BRIDGE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct {
    uint32_t count;
    int32_t min;
    int32_t max;
    int32_t sum;
    uint32_t high_count;
} E4Stats;
uint32_t e4_abi_version(void);
int32_t e4_classify(int32_t value, int32_t low, int32_t high);
/* Caller owns disjoint readable samples and writable output until return.
   len is 1..4096; null/length/alignment errors are rejected.
   Nonzero status leaves output unchanged. Invalid non-null addresses are UB. */
int32_t e4_summarize(const int32_t *samples, uint32_t len, int32_t low, int32_t high, E4Stats *out);
#ifdef __cplusplus
}
#endif
#endif
