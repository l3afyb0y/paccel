#ifndef __PACCEL_TELEMETRY_K_H__
#define __PACCEL_TELEMETRY_K_H__

#include "fixedptc.h"
#include <linux/atomic.h>

static atomic64_t PACCEL_LAST_INPUT_SPEED = ATOMIC64_INIT(0);

static inline void paccel_publish_input_speed(fpt speed) {
  atomic64_set(&PACCEL_LAST_INPUT_SPEED, speed);
}

static inline fpt paccel_last_input_speed(void) {
  return (fpt)atomic64_read(&PACCEL_LAST_INPUT_SPEED);
}

#endif
