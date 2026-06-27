#define NDEBUG

#include "ini.c"

static int streq(const char* left, const char* right)
{
    while (*left && *right && *left == *right) {
        left++;
        right++;
    }
    return *left == *right;
}

static int seen_host;
static int seen_port;
static int seen_enabled;
static int seen_user;

static int handler(void* user, const char* section, const char* name, const char* value)
{
    int* errors = (int*)user;

    if (streq(section, "server") && streq(name, "host") && streq(value, "localhost")) {
        seen_host = 1;
        return 1;
    }

    if (streq(section, "server") && streq(name, "port") && streq(value, "8080")) {
        seen_port = 1;
        return 1;
    }

    if (streq(section, "server") && streq(name, "enabled") && streq(value, "true")) {
        seen_enabled = 1;
        return 1;
    }

    if (streq(section, "client") && streq(name, "user") && streq(value, "kteal")) {
        seen_user = 1;
        return 1;
    }

    *errors = *errors + 1;
    return 1;
}

int main(void)
{
    int errors = 0;
    int parse_error = ini_parse_string(
        "; comment before first section\n"
        "[server]\n"
        "host = localhost\n"
        "port: 8080\n"
        "enabled = true ; inline comment\n"
        "\n"
        "[client]\n"
        "user = kteal\n",
        handler,
        &errors);

    if (parse_error != 0) {
        return 1;
    }

    if (errors != 0) {
        return 2;
    }

    if (!seen_host) {
        return 3;
    }

    if (!seen_port) {
        return 4;
    }

    if (!seen_enabled) {
        return 5;
    }

    if (!seen_user) {
        return 6;
    }

    return 0;
}
