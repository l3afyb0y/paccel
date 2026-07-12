#ifndef __PACCEL_CONFIG_K_H__
#define __PACCEL_CONFIG_K_H__

#include "config.h"
#include <linux/spinlock.h>

static DEFINE_RWLOCK(PACCEL_CONFIG_LOCK);
static struct paccel_config_v1 PACCEL_ACTIVE_CONFIG;

static inline void paccel_config_store_init(void) {
  PACCEL_ACTIVE_CONFIG = paccel_default_config();
}

static inline struct paccel_config_v1 paccel_config_snapshot(void) {
  struct paccel_config_v1 snapshot;

  read_lock(&PACCEL_CONFIG_LOCK);
  snapshot = PACCEL_ACTIVE_CONFIG;
  read_unlock(&PACCEL_CONFIG_LOCK);
  return snapshot;
}

static inline int
paccel_config_replace(const struct paccel_config_v1 *candidate) {
  int error = paccel_validate_config(candidate);
  if (error)
    return error;

  write_lock(&PACCEL_CONFIG_LOCK);
  PACCEL_ACTIVE_CONFIG = *candidate;
  write_unlock(&PACCEL_CONFIG_LOCK);
  return 0;
}

#endif
