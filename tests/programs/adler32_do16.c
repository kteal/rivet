#define DO1(buf, i) { adler += (buf)[i]; sum2 += adler; }
#define DO2(buf, i) DO1(buf, i); DO1(buf, i + 1);
#define DO4(buf, i) DO2(buf, i); DO2(buf, i + 2);
#define DO8(buf, i) DO4(buf, i); DO4(buf, i + 4);
#define DO16(buf) DO8(buf, 0); DO8(buf, 8);

int main() {
    unsigned char buf[16] = {
        1, 2, 3, 4,
        5, 6, 7, 8,
        9, 10, 11, 12,
        13, 14, 15, 16,
    };
    unsigned long adler = 1;
    unsigned long sum2 = 0;

    DO16(buf);

    if (adler != 137) {
        return 1;
    }

    if (sum2 != 832) {
        return 2;
    }

    return 0;
}
