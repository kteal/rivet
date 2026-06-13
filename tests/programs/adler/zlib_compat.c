#define BASE 65521U
#define Z_NULL 0
#define ZEXPORT
#define local

typedef unsigned int uInt;
typedef unsigned long uLong;
typedef unsigned int z_size_t;
typedef long z_off_t;
typedef long z_off64_t;
typedef unsigned char Bytef;

uLong ZEXPORT adler32_z(uLong adler, const Bytef *buf, z_size_t len) {
    uLong sum2 = (adler >> 16) & 0xffffUL;

    adler &= 0xffffUL;

    if (buf == Z_NULL) {
        return 1L;
    }

    if (len == 1U) {
        adler += buf[0];

        if (adler >= BASE) {
            adler -= BASE;
        }

        sum2 += adler;

        if (sum2 >= BASE) {
            sum2 -= BASE;
        }

        return adler | (sum2 << 16);
    }

    if (len < 16U) {
        while (len) {
            adler += *buf++;
            sum2 += adler;
            len--;
        }

        if (adler >= BASE) {
            adler -= BASE;
        }

        sum2 %= BASE;

        return adler | (sum2 << 16);
    }

    return 0UL;
}

uLong ZEXPORT adler32(uLong adler, const Bytef *buf, uInt len) {
    return adler32_z(adler, buf, len);
}

int main() {
    Bytef buf[3] = {'a', 'b', 'c'};

    if (adler32(1L, Z_NULL, 3U) != 1L) {
        return 1;
    }

    if (adler32(1L, buf, 1U) != 6422626L) {
        return 2;
    }

    if (adler32(1L, buf, 3U) != 38600999L) {
        return 3;
    }

    return 0;
}
