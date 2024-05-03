
#include "ers.h"
#include <stdlib.h>


Ers_String_View command(Ers_Api *api, Ers_String_View *args, size_t args_len, void *user_data) {
    (void) api;
    (void) args;
    (void) args_len;
    (void) user_data;

    Ers_String_View msg = ERS_SV("Hello, World!");

    Ers_String_View row = api->get_curr_row(api->editor);
    {
        char *buf = malloc((row.len + msg.len)*sizeof(char));
        memcpy(buf, row.data, row.len);
        memcpy(buf + row.len, msg.data, msg.len);
        api->update_curr_row(api->editor, (Ers_String_View) { .data = buf, .len = row.len + msg.len });
        free(buf);
    }

    return ERS_SV_EMPTY;
}

void ers_plugin_init(Ers_Api *api) {
    api->add_cmd(api->plugin, ERS_SV("hello"), command, NULL);
}

