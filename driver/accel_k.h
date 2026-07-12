#ifndef _ACCELK_H_
#define _ACCELK_H_

#include "accel.h"
#include "config_k.h"
#include "telemetry_k.h"
#include "timing.h"
#include <linux/ktime.h>

struct paccel_motion_state {
  struct accel_state accel;
  ktime_t last_time;
  s64 expected_interval_ns;
};

static inline void paccel_accelerate(struct paccel_motion_state *state, int *x,
                                    int *y) {
  struct paccel_config_v1 config = paccel_config_snapshot();
  struct accel_args args = paccel_config_to_accel_args(&config);
  ktime_t now = ktime_get();
  s64 elapsed_ns = 0;

  if (state->last_time)
    elapsed_ns = ktime_to_ns(ktime_sub(now, state->last_time));
  state->last_time = now;

  if (elapsed_ns >= PACCEL_MIN_INTERVAL_NS &&
      elapsed_ns <= PACCEL_MAX_INTERVAL_NS) {
    if (!state->expected_interval_ns)
      state->expected_interval_ns = elapsed_ns;
    else
      state->expected_interval_ns =
          (state->expected_interval_ns * 7 + elapsed_ns) / 8;
  }

  f_accelerate(x, y,
               paccel_interval_ms(elapsed_ns, state->expected_interval_ns),
               args, &state->accel);
  paccel_publish_input_speed(state->accel.last_input_speed);
}

#endif // !_ACCELK_H_
