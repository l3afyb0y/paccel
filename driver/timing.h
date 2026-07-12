#ifndef __PACCEL_TIMING_H__
#define __PACCEL_TIMING_H__

#include "fixedptc.h"

#ifdef __KERNEL__
#include <linux/types.h>
typedef s64 paccel_s64;
#else
#include <stdint.h>
typedef int64_t paccel_s64;
#endif

#define PACCEL_DEFAULT_INTERVAL_NS 1000000LL
#define PACCEL_MIN_INTERVAL_NS 50000LL
#define PACCEL_MAX_INTERVAL_NS 20000000LL

static inline paccel_s64 paccel_valid_fallback_ns(paccel_s64 fallback_ns) {
  if (fallback_ns < PACCEL_MIN_INTERVAL_NS ||
      fallback_ns > PACCEL_MAX_INTERVAL_NS)
    return PACCEL_DEFAULT_INTERVAL_NS;
  return fallback_ns;
}

static inline fpt paccel_interval_ms(paccel_s64 elapsed_ns,
                                     paccel_s64 fallback_ns) {
  paccel_s64 interval_ns = elapsed_ns;

  if (interval_ns < PACCEL_MIN_INTERVAL_NS ||
      interval_ns > PACCEL_MAX_INTERVAL_NS)
    interval_ns = paccel_valid_fallback_ns(fallback_ns);

  return (fpt)(((fptd)interval_ns << FIXEDPT_FBITS) / 1000000);
}

#endif
