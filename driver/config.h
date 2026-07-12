#ifndef __PACCEL_CONFIG_H__
#define __PACCEL_CONFIG_H__

#include "accel.h"

#ifdef __KERNEL__
#include <linux/errno.h>
#include <linux/types.h>
typedef u16 paccel_u16;
typedef u8 paccel_u8;
#else
#include <errno.h>
#include <stdint.h>
typedef uint16_t paccel_u16;
typedef uint8_t paccel_u8;
#endif

#define PACCEL_ABI_VERSION 1

struct paccel_config_v1 {
  paccel_u16 abi_version;
  paccel_u16 struct_size;
  paccel_u8 mode;
  paccel_u8 reserved[3];

  fpt sens_mult;
  fpt yx_ratio;
  fpt input_dpi;
  fpt angle_rotation;

  fpt accel;
  fpt offset;
  fpt output_cap;

  fpt decay_rate;
  fpt limit;

  fpt gamma;
  fpt smooth;
  fpt motivity;
  fpt sync_speed;
};

static inline struct paccel_config_v1 paccel_default_config(void) {
  return (struct paccel_config_v1){
      .abi_version = PACCEL_ABI_VERSION,
      .struct_size = sizeof(struct paccel_config_v1),
      .mode = linear,
      .sens_mult = FIXEDPT_ONE,
      .yx_ratio = FIXEDPT_ONE,
      .input_dpi = fpt_fromint(1000),
      .angle_rotation = 0,
      .accel = 0,
      .offset = 0,
      .output_cap = 0,
      .decay_rate = fpt_rconst(0.1),
      .limit = fpt_rconst(1.5),
      .gamma = FIXEDPT_ONE,
      .smooth = fpt_rconst(0.5),
      .motivity = fpt_rconst(1.5),
      .sync_speed = fpt_fromint(5),
  };
}

static inline int
paccel_validate_config(const struct paccel_config_v1 *config) {
  if (!config || config->abi_version != PACCEL_ABI_VERSION ||
      config->struct_size != sizeof(*config))
    return -EINVAL;

  if (config->mode > no_accel || config->sens_mult <= 0 ||
      config->yx_ratio <= 0 || config->input_dpi <= 0)
    return -EINVAL;

  switch ((enum accel_mode)config->mode) {
  case linear:
    if (config->offset < 0 || config->accel < 0)
      return -EINVAL;
    break;
  case natural:
    if (config->offset < 0 || config->decay_rate <= 0 ||
        config->limit < FIXEDPT_ONE)
      return -EINVAL;
    break;
  case synchronous:
    if (config->gamma <= 0 || config->smooth < 0 ||
        config->smooth > FIXEDPT_ONE || config->motivity <= FIXEDPT_ONE ||
        config->sync_speed <= 0)
      return -EINVAL;
    break;
  case no_accel:
    break;
  }

  return 0;
}

static inline struct accel_args
paccel_config_to_accel_args(const struct paccel_config_v1 *config) {
  struct accel_args args = {
      .sens_mult = config->sens_mult,
      .yx_ratio = config->yx_ratio,
      .input_dpi = config->input_dpi,
      .angle_rotation_deg = config->angle_rotation,
      .tag = (enum accel_mode)config->mode,
  };

  switch (args.tag) {
  case linear:
    args.args.linear = (struct linear_curve_args){
        .accel = config->accel,
        .offset = config->offset,
        .output_cap = config->output_cap,
    };
    break;
  case natural:
    args.args.natural = (struct natural_curve_args){
        .decay_rate = config->decay_rate,
        .offset = config->offset,
        .limit = config->limit,
    };
    break;
  case synchronous:
    args.args.synchronous = (struct synchronous_curve_args){
        .gamma = config->gamma,
        .smooth = config->smooth,
        .motivity = config->motivity,
        .sync_speed = config->sync_speed,
    };
    break;
  case no_accel:
    break;
  }

  return args;
}

#endif
