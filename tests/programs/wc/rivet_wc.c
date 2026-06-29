#include <stdio.h>

struct Counts {
    int lines;
    int words;
    int bytes;
};

struct Options {
    int print_lines;
    int print_words;
    int print_bytes;
};

static int is_space(int c)
{
    return c == ' ' || c == '\n' || c == '\t' || c == '\r' || c == 12 || c == 11;
}

static void add_counts(struct Counts* total, struct Counts* counts)
{
    total->lines = total->lines + counts->lines;
    total->words = total->words + counts->words;
    total->bytes = total->bytes + counts->bytes;
}

static void clear_counts(struct Counts* counts)
{
    counts->lines = 0;
    counts->words = 0;
    counts->bytes = 0;
}

static void default_options(struct Options* options)
{
    if (!options->print_lines && !options->print_words && !options->print_bytes) {
        options->print_lines = 1;
        options->print_words = 1;
        options->print_bytes = 1;
    }
}

static int parse_options(int argc, char** argv, struct Options* options)
{
    int i = 1;

    options->print_lines = 0;
    options->print_words = 0;
    options->print_bytes = 0;

    while (i < argc && argv[i][0] == '-' && argv[i][1] != 0) {
        char* p = argv[i] + 1;

        while (*p) {
            if (*p == 'l') {
                options->print_lines = 1;
            } else if (*p == 'w') {
                options->print_words = 1;
            } else if (*p == 'c') {
                options->print_bytes = 1;
            } else {
                return -1;
            }

            p = p + 1;
        }

        i = i + 1;
    }

    default_options(options);
    return i;
}

static void print_int(int value)
{
    char digits[16];
    int count = 0;

    if (value == 0) {
        putchar('0');
        return;
    }

    while (value > 0) {
        digits[count] = '0' + value % 10;
        count = count + 1;
        value = value / 10;
    }

    while (count > 0) {
        count = count - 1;
        putchar(digits[count]);
    }
}

static void print_string(const char* s)
{
    while (*s) {
        putchar(*s);
        s = s + 1;
    }
}

static void print_selected_count(int enabled, int value, int* printed)
{
    if (enabled) {
        if (*printed) {
            putchar(' ');
        }

        print_int(value);
        *printed = 1;
    }
}

static void print_counts(struct Counts* counts, struct Options* options, const char* path)
{
    int printed = 0;

    print_selected_count(options->print_lines, counts->lines, &printed);
    print_selected_count(options->print_words, counts->words, &printed);
    print_selected_count(options->print_bytes, counts->bytes, &printed);

    if (path != 0) {
        if (printed) {
            putchar(' ');
        }

        print_string(path);
    }

    putchar('\n');
}

static int count_file(const char* path, struct Counts* counts)
{
    FILE* file = fopen(path, "r");
    int in_word = 0;
    int c;

    if (file == 0) {
        return 1;
    }

    clear_counts(counts);

    c = fgetc(file);
    while (c != EOF) {
        counts->bytes = counts->bytes + 1;

        if (c == '\n') {
            counts->lines = counts->lines + 1;
        }

        if (is_space(c)) {
            in_word = 0;
        } else if (!in_word) {
            counts->words = counts->words + 1;
            in_word = 1;
        }

        c = fgetc(file);
    }

    if (fclose(file) != 0) {
        return 2;
    }

    return 0;
}

int main(int argc, char** argv)
{
    const char* default_path = "tests/programs/wc/sample.txt";
    struct Options options;
    struct Counts counts;
    struct Counts total;
    int first_file = parse_options(argc, argv, &options);
    int files_seen = 0;
    int had_error = 0;
    int i;

    if (first_file < 0) {
        return 2;
    }

    clear_counts(&total);

    if (first_file == argc) {
        int err = count_file(default_path, &counts);
        if (err != 0) {
            return err;
        }

        print_counts(&counts, &options, default_path);
        return 0;
    }

    i = first_file;
    while (i < argc) {
        int err = count_file(argv[i], &counts);
        if (err != 0) {
            had_error = 1;
        } else {
            print_counts(&counts, &options, argv[i]);
            add_counts(&total, &counts);
            files_seen = files_seen + 1;
        }

        i = i + 1;
    }

    if (files_seen > 1) {
        print_counts(&total, &options, "total");
    }

    return had_error;
}
