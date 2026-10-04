#include <stdio.h>
#include <string.h>

int main(void) {
    static char buf[65536];
    int off = 0;
    off += sprintf(buf + off, "[");
    for (int i = 0; i < 1000; i++) {
        off += sprintf(buf + off, "%s{\"id\":%d,\"name\":\"item\"}",
                       i == 0 ? "" : ",", i);
    }
    off += sprintf(buf + off, "]");
    /* parse simplificado: conta ocorrencias de "id" */
    int count = 0;
    const char *p = buf;
    while ((p = strstr(p, "\"id\"")) != NULL) {
        count++;
        p += 4;
    }
    /* stringify = o proprio buffer */
    printf("count=%d len=%d\n", count, (int)strlen(buf));
    return 0;
}
