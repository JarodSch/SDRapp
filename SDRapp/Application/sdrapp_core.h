#ifndef SDRAPP_CORE_H
#define SDRAPP_CORE_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define FFT_SIZE 1024

typedef struct SdrappCore SdrappCore;

typedef struct DeviceItemC {
  char *label;
  char *args;
} DeviceItemC;

typedef struct DeviceListC {
  uintptr_t count;
  struct DeviceItemC *items;
} DeviceListC;

struct SdrappCore *sdrapp_create(void);

void sdrapp_destroy(struct SdrappCore *ptr);

void sdrapp_set_device(struct SdrappCore *ptr, const char *args);

void sdrapp_set_frequency(struct SdrappCore *ptr, uint64_t hz);

void sdrapp_set_gain(struct SdrappCore *ptr, double db);

void sdrapp_set_demod(struct SdrappCore *ptr, uint32_t mode);

bool sdrapp_start(struct SdrappCore *ptr);

void sdrapp_stop(struct SdrappCore *ptr);

/**
 * Kopiert FFT-Daten in out_buf. Gibt Anzahl geschriebener Werte zurück.
 * out_len muss >= 1024 sein.
 */
uintptr_t sdrapp_get_fft(const struct SdrappCore *ptr, float *out_buf, uintptr_t out_len);

/**
 * Gibt Anzahl angeschlossener Geräte zurück.
 */
struct DeviceListC *sdrapp_list_devices(uintptr_t *out_count);

void sdrapp_free_device_list(struct DeviceListC *ptr);

#endif  /* SDRAPP_CORE_H */
