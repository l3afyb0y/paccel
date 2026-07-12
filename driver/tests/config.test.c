#include "../config.h"
#include <assert.h>
#include <stdio.h>

int main(void) {
  struct paccel_config_v1 config = paccel_default_config();

  assert(paccel_validate_config(&config) == 0);

  config.input_dpi = 0;
  assert(paccel_validate_config(&config) != 0);

  config = paccel_default_config();
  config.mode = 255;
  assert(paccel_validate_config(&config) != 0);

  config = paccel_default_config();
  config.mode = synchronous;
  config.motivity = FIXEDPT_ONE;
  assert(paccel_validate_config(&config) != 0);

  config = paccel_default_config();
  config.abi_version++;
  assert(paccel_validate_config(&config) != 0);

  puts("[config.test.c]\tTyped configuration validation passed");
  return 0;
}
