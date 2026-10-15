#include <stdio.h>
#include <stdint.h>

__attribute__((weak)) unsigned long getauxval(unsigned long type) {
    FILE *file = fopen("/proc/self/auxv", "rb");
    if (!file) return 0;
    unsigned long entry[2];
    unsigned long value = 0;
    while (fread(entry, sizeof(entry), 1, file) == 1 && entry[0] != 0) {
        if (entry[0] == type) {
            value = entry[1];
            break;
        }
    }
    fclose(file);
    return value;
}
