#include "signal_bridge.h"
#include <assert.h>
#include <stdio.h>
int main(void) {
    const int32_t samples[] = {-4, 10, 19, 20, 31};
    E4Stats result = {99, -1, -1, -1, 99};
    assert(sizeof(E4Stats) == 20);
    assert(e4_abi_version() == 1);
    assert(e4_classify(9, 10, 20) == 0);
    assert(e4_classify(10, 10, 20) == 1);
    assert(e4_classify(20, 10, 20) == 2);
    assert(e4_summarize(samples, 5, 10, 20, &result) == 0);
    assert(result.count == 5 && result.min == -4 && result.max == 31);
    assert(result.sum == 76 && result.high_count == 2);
    assert(e4_summarize(samples, 5, 20, 20, &result) == -1);
    assert(result.count == 5 && result.min == -4 && result.max == 31);
    assert(result.sum == 76 && result.high_count == 2);
    assert(e4_summarize(NULL, 5, 10, 20, &result) == -3);
    assert(e4_summarize(samples, 0, 10, 20, &result) == -2);
    puts("E4_C_ABI_PASS");
    return 0;
}
