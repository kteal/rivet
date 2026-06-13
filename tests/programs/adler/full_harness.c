#include "adler32.c"

int main() {
    Bytef abc[3] = {'a', 'b', 'c'};
    Bytef sixteen[16] = {
        1, 2, 3, 4,
        5, 6, 7, 8,
        9, 10, 11, 12,
        13, 14, 15, 16,
    };

    if (adler32(1L, Z_NULL, 3U) != 1L) {
        return 1;
    }

    if (adler32(1L, abc, 1U) != 6422626L) {
        return 2;
    }

    if (adler32(1L, abc, 3U) != 38600999L) {
        return 3;
    }

    if (adler32(1L, sixteen, 16U) != 54526089L) {
        return 4;
    }

    return 0;
}
