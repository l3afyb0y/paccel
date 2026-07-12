#ifndef __PACCEL_INPUT_HANDLER_H__
#define __PACCEL_INPUT_HANDLER_H__

#include "accel_k.h"
#include <linux/hid.h>
#include <linux/input.h>
#include <linux/slab.h>
#include <linux/string.h>
#include <linux/version.h>

#if LINUX_VERSION_CODE < KERNEL_VERSION(6, 11, 0)
#error "paccel requires Linux 6.11 or newer"
#endif

struct paccel_device {
  struct input_handle handle;
  struct paccel_motion_state motion;
  struct input_value *scratch;
  unsigned int scratch_capacity;
};

static inline struct paccel_device *
paccel_device_from_handle(struct input_handle *handle) {
  return container_of(handle, struct paccel_device, handle);
}

static unsigned int
paccel_transform_frame(struct paccel_device *device, struct input_value *values,
                       unsigned int count, unsigned int frame_start,
                       unsigned int syn_index) {
  int x = 0;
  int y = 0;
  int first_x = -1;
  int first_y = -1;
  unsigned int index;

  for (index = frame_start; index < syn_index; index++) {
    struct input_value *value = &values[index];
    if (value->type != EV_REL)
      continue;

    if (value->code == REL_X) {
      x += value->value;
      if (first_x < 0)
        first_x = index;
      else
        value->value = 0;
    } else if (value->code == REL_Y) {
      y += value->value;
      if (first_y < 0)
        first_y = index;
      else
        value->value = 0;
    }
  }

  if (!x && !y)
    return count;

  paccel_accelerate(&device->motion, &x, &y);

  if (first_x >= 0) {
    values[first_x].value = x;
  } else if (x && count < device->scratch_capacity) {
    memmove(&values[syn_index + 1], &values[syn_index],
            (count - syn_index) * sizeof(*values));
    values[syn_index] =
        (struct input_value){.type = EV_REL, .code = REL_X, .value = x};
    syn_index++;
    count++;
  }

  if (first_y >= 0) {
    values[first_y].value = y;
  } else if (y && count < device->scratch_capacity) {
    memmove(&values[syn_index + 1], &values[syn_index],
            (count - syn_index) * sizeof(*values));
    values[syn_index] =
        (struct input_value){.type = EV_REL, .code = REL_Y, .value = y};
    count++;
  }

  return count;
}

static unsigned int paccel_events(struct input_handle *handle,
                                  struct input_value *values,
                                  unsigned int count) {
  struct paccel_device *device = paccel_device_from_handle(handle);
  unsigned int frame_start = 0;
  unsigned int index = 0;

  if (count > device->scratch_capacity)
    return count;

  memcpy(device->scratch, values, count * sizeof(*values));

  while (index < count) {
    struct input_value *value = &device->scratch[index];
    if (value->type == EV_SYN && value->code == SYN_REPORT) {
      unsigned int previous_count = count;
      count = paccel_transform_frame(device, device->scratch, count,
                                     frame_start, index);
      index += count - previous_count;
      frame_start = index + 1;
    }
    index++;
  }

  memcpy(values, device->scratch, count * sizeof(*values));
  return count;
}

/* Register ahead of evdev so later handlers observe the transformed buffer. */
static int paccel_register_handle_head(struct input_handle *handle) {
  struct input_handler *handler = handle->handler;
  struct input_dev *dev = handle->dev;
  int error;

#if LINUX_VERSION_CODE >= KERNEL_VERSION(6, 11, 7)
  if (handler->events)
    handle->handle_events = handler->events;
#endif

  error = mutex_lock_interruptible(&dev->mutex);
  if (error)
    return error;

  list_add_rcu(&handle->d_node, &dev->h_list);
  mutex_unlock(&dev->mutex);
  list_add_tail_rcu(&handle->h_node, &handler->h_list);
  if (handler->start)
    handler->start(handle);
  return 0;
}

static bool paccel_match(struct input_handler *handler, struct input_dev *dev) {
  bool has_axes = test_bit(REL_X, dev->relbit) && test_bit(REL_Y, dev->relbit);
  bool is_pointer = test_bit(BTN_MOUSE, dev->keybit) ||
                    test_bit(INPUT_PROP_POINTER, dev->propbit) ||
                    test_bit(INPUT_PROP_DIRECT, dev->propbit);

  return has_axes && is_pointer;
}

static int paccel_connect(struct input_handler *handler, struct input_dev *dev,
                          const struct input_device_id *id) {
  struct paccel_device *device;
  int error;

  device = kzalloc(sizeof(*device), GFP_KERNEL);
  if (!device)
    return -ENOMEM;

  device->scratch_capacity = dev->max_vals;
  device->scratch =
      kcalloc(device->scratch_capacity, sizeof(*device->scratch), GFP_KERNEL);
  if (!device->scratch) {
    error = -ENOMEM;
    goto err_free_device;
  }

  device->handle.dev = input_get_device(dev);
  device->handle.handler = handler;
  device->handle.name = "paccel";

  error = paccel_register_handle_head(&device->handle);
  if (error)
    goto err_put_input;

  error = input_open_device(&device->handle);
  if (error)
    goto err_unregister_handle;

  pr_info("paccel connected to %s (%s at %s)\n", dev_name(&dev->dev),
          dev->name ?: "unknown", dev->phys ?: "unknown");
  return 0;

err_unregister_handle:
  input_unregister_handle(&device->handle);
err_put_input:
  input_put_device(device->handle.dev);
  kfree(device->scratch);
err_free_device:
  kfree(device);
  return error;
}

static void paccel_disconnect(struct input_handle *handle) {
  struct paccel_device *device = paccel_device_from_handle(handle);

  input_close_device(handle);
  input_unregister_handle(handle);
  input_put_device(handle->dev);
  kfree(device->scratch);
  kfree(device);
}

static const struct input_device_id paccel_ids[] = {
    {.flags = INPUT_DEVICE_ID_MATCH_EVBIT,
     .evbit = {BIT_MASK(EV_REL)}},
    {},
};

MODULE_DEVICE_TABLE(input, paccel_ids);

static struct input_handler paccel_handler = {
    .events = paccel_events,
    .match = paccel_match,
    .connect = paccel_connect,
    .disconnect = paccel_disconnect,
    .name = "paccel",
    .id_table = paccel_ids,
};

#endif
