#include "../accel.h"
#include <assert.h>
#include <stdio.h>

static struct accel_args half_sensitivity(void) {
  return (struct accel_args){
      .sens_mult = fpt_rconst(0.5),
      .yx_ratio = FIXEDPT_ONE,
      .input_dpi = fpt_fromint(1000),
      .tag = no_accel,
      .args = (union __accel_args){.no_accel = {}}};
}

int main(void) {
  struct accel_state first = {0};
  struct accel_state second = {0};
  struct accel_args args = half_sensitivity();
  int x = 1;
  int y = 0;

  f_accelerate(&x, &y, FIXEDPT_ONE, args, &first);
  assert(x == 0);

  x = 1;
  f_accelerate(&x, &y, FIXEDPT_ONE, args, &second);
  assert(x == 0);

  x = 1;
  f_accelerate(&x, &y, FIXEDPT_ONE, args, &first);
  assert(x == 1);

  puts("[state.test.c]\t\tPer-device carry is isolated");
  return 0;
}
