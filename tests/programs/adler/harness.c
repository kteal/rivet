unsigned long adler32_current(unsigned long adler, unsigned char *buf, unsigned int len) {
    unsigned long sum2 = (adler >> 16) & 0xffffUL;

    adler &= 0xffffUL;

    if (buf == 0) {
        return 1L;
    }

    if (len == 1U) {
        adler += buf[0];

        if (adler >= 65521U) {
            adler -= 65521U;
        }

        sum2 += adler;

        if (sum2 >= 65521U) {
            sum2 -= 65521U;
        }

        return adler | (sum2 << 16);
    }

    if (len < 16U) {
        while (len) {
            adler += *buf++;
            sum2 += adler;
            len--;
        }

        if (adler >= 65521U) {
            adler -= 65521U;
        }

        sum2 %= 65521U;

        return adler | (sum2 << 16);
    }

    return 0UL;
}

int main() {
    unsigned char buf[3] = {'a', 'b', 'c'};

    if (adler32_current(1L, 0, 3U) != 1L) {
        return 1;
    }

    if (adler32_current(1L, buf, 1U) != 6422626L) {
        return 2;
    }

    if (adler32_current(1L, buf, 3U) != 38600999L) {
        return 3;
    }

    return 0;
}
