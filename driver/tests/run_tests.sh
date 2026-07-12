#!/bin/bash

for test in tests/*.test.c; do
  if [[ ! $test =~ $TEST_NAME ]]; then
    continue
  fi
  gcc ${test} -o paccel_test -lm $DRIVER_CFLAGS || exit 1
  ./paccel_test || exit 1
  rm paccel_test
done
