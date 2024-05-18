use std::ffi::CString;

use crate::CursorState;

#[repr(C)]
pub struct StringView<'a> {
    data: *const u8,
    len: usize,
    _p: std::marker::PhantomData<&'a str>,
}

impl<'a> From<StringView<'a>> for &'a [u8] {
    fn from(val: StringView<'a>) -> Self {
        unsafe { std::slice::from_raw_parts(val.data, val.len) }
    }
}

impl<'a> From<&'a [u8]> for StringView<'a> {
    fn from(s: &'a [u8]) -> Self {
        Self {
            data: s.as_ptr(),
            len: s.len(),
            _p: std::marker::PhantomData,
        }
    }
}

impl<'a> From<StringView<'a>> for String {
    fn from(val: StringView<'a>) -> Self {
        unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(val.data, val.len))
                .unwrap()
                .into()
        }
    }
}

impl<'a> From<&'a str> for StringView<'a> {
    fn from(s: &'a str) -> Self {
        Self {
            data: s.as_ptr(),
            len: s.len(),
            _p: std::marker::PhantomData,
        }
    }
}

pub type CmdCallback =
    extern "C" fn(*mut Api, *const StringView, usize, *mut libc::c_void) -> StringView;

#[repr(C)]
pub struct Api {
    editor: *mut crate::Editor,
    plugin: *mut Plugin,
    is_cursor_in_status: bool,
    set_status: unsafe extern "C" fn(*mut crate::Editor, StringView),
    add_cmd: unsafe extern "C" fn(*mut Plugin, StringView, CmdCallback, *mut libc::c_void),
    get_curr_row: unsafe extern "C" fn(*mut crate::Editor) -> StringView<'static>,
    update_curr_row: unsafe extern "C" fn(*mut crate::Editor, StringView),
    on_render: unsafe extern "C" fn(
        *mut Plugin,
        extern "C" fn(*mut Api, *mut libc::c_void),
        *mut libc::c_void,
    ),
}

impl Api {
    pub unsafe fn new(editor: &mut crate::Editor, plugin: *mut Plugin) -> Self {
        unsafe extern "C" fn set_status(editor: *mut crate::Editor, status: StringView) {
            (*editor).set_status(status.into())
        }
        unsafe extern "C" fn add_cmd(
            plugin: *mut Plugin,
            cmd: StringView,
            callback: CmdCallback,
            data: *mut libc::c_void,
        ) {
            (*plugin).add_cmd(cmd.into(), callback, data)
        }
        unsafe extern "C" fn get_curr_row(_editor: *mut crate::Editor) -> StringView<'static> {
            todo!()
            //(*editor).row().into_iter().map(|ch| *ch).collect::<String>().as_bytes().into()
        }
        unsafe extern "C" fn update_curr_row(editor: *mut crate::Editor, new_row: StringView) {
            let new_row: &[u8] = new_row.into();
            (*editor).log(format!("Updating current row: {:?}", new_row));
            let row = (*editor).row();
            row.clear();
            row.extend_from_slice(
                &std::str::from_utf8(new_row)
                    .unwrap()
                    .chars()
                    .collect::<Vec<char>>(),
            );
        }
        unsafe extern "C" fn on_render(
            plugin: *mut Plugin,
            callback: extern "C" fn(*mut Api, *mut libc::c_void),
            data: *mut libc::c_void,
        ) {
            (*plugin).on_render = Some((callback, data));
        }
        Self {
            editor: editor as *mut _ as _,
            plugin,
            is_cursor_in_status: !matches!(editor.cursor.state, CursorState::Default),
            set_status,
            add_cmd,
            get_curr_row,
            update_curr_row,
            on_render,
        }
    }
}

pub struct Plugin {
    handle: *mut libc::c_void,
    pub init: extern "C" fn(*mut Api),
    pub cmds: Vec<(String, CmdCallback, *mut libc::c_void)>,
    pub on_render: Option<(
        extern "C" fn(*mut Api, *mut libc::c_void),
        *mut libc::c_void,
    )>,
}

impl Plugin {
    pub fn load(path: String) -> Result<Self, CString> {
        let handle = unsafe { libc::dlopen(path.as_ptr() as _, libc::RTLD_NOW) };
        if handle.is_null() {
            let msg = unsafe { CString::from_raw(libc::dlerror()) };
            return Err(msg);
        }

        let init = unsafe { libc::dlsym(handle, c"ers_plugin_init".as_ptr()) };
        if init.is_null() {
            let msg = unsafe { CString::from_raw(libc::dlerror()) };
            return Err(msg);
        }

        Ok(Self {
            handle,
            init: unsafe { std::mem::transmute(init) },
            cmds: Vec::new(),
            on_render: None,
        })
    }

    pub fn add_cmd(&mut self, cmd: String, callback: CmdCallback, data: *mut libc::c_void) {
        self.cmds.push((cmd, callback, data));
    }
}

impl std::ops::Drop for Plugin {
    fn drop(&mut self) {
        unsafe {
            libc::dlclose(self.handle);
        }
    }
}
