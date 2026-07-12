#include "../timing.h"
#include <assert.h>
#include <stdio.h>

int main(void) {
  const int64_t fallback_ns = 1000000;

  assert(paccel_interval_ms(0, fallback_ns) == fpt_rconst(1.0));
  assert(paccel_interval_ms(3000000000LL, fallback_ns) ==
         fpt_rconst(1.0));
  assert(paccel_interval_ms(125000, fallback_ns) == fpt_rconst(0.125));
  assert(paccel_interval_ms(1000000, fallback_ns) == fpt_rconst(1.0));

  puts("[timing.test.c]\tSafe interval conversion passed");
  return 0;
}
