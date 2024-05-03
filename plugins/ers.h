#ifndef ERS_H
#define ERS_H

#include <stddef.h>
#include <string.h>

typedef struct {
    char *data;
    size_t len;
} Ers_String_View;

#define ERS_SV(cstr) ((Ers_String_View) { .data = (cstr), .len = strlen(cstr) })
#define ERS_SV_EMPTY ((Ers_String_View) { .data = NULL, .len = 0 })

typedef struct Ers_Api {
    void *editor;
    void *plugin;
    void (*set_status)(void *editor, Ers_String_View status);
    void (*add_cmd)
        (
            void *plugin,
            Ers_String_View cmd,
            Ers_String_View (*callback)(struct Ers_Api *api, Ers_String_View *args, size_t args_len, void *user_data),
            void *user_data
        );
    Ers_String_View (*get_curr_row)(void *editor);
    void (*update_curr_row)(void *editor, Ers_String_View row);
} Ers_Api;


#endif // ERS_H
