use libc;

use std::ffi::CString;

#[repr(C)]
pub struct StringView<'a> {
    data: *const u8,
    len: usize,
    _p: std::marker::PhantomData<&'a str>,
}

impl<'a> Into<&'a [u8]> for StringView<'a> {
    fn into(self) -> &'a [u8] {
        unsafe {
            std::slice::from_raw_parts(self.data, self.len)
        }
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

impl<'a> Into<String> for StringView<'a> {
    fn into(self) -> String {
        unsafe {
            std::str::from_utf8(std::slice::from_raw_parts(self.data, self.len)).unwrap().into()
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

pub type CmdCallback = extern "C" fn(*mut Api, *const StringView, usize, *mut libc::c_void) -> StringView;

#[repr(C)]
pub struct Api {
    editor: *mut crate::Editor,
    plugin: *mut Plugin,
    set_status: unsafe extern "C" fn(*mut crate::Editor, StringView),
    add_cmd: unsafe extern "C" fn(*mut Plugin, StringView, CmdCallback, *mut libc::c_void),
    get_curr_row: unsafe extern "C" fn(*mut crate::Editor) -> StringView<'static>,
    update_curr_row: unsafe extern "C" fn(*mut crate::Editor, StringView),
}

impl Api {
    pub unsafe fn new(editor: &mut crate::Editor, plugin: *mut Plugin) -> Self {
        unsafe extern "C" fn set_status(editor: *mut crate::Editor, status: StringView) {
            (*editor).set_status(status.into())
        }
        unsafe extern "C" fn add_cmd(plugin: *mut Plugin, cmd: StringView, callback: CmdCallback, data: *mut libc::c_void) {
            (*plugin).add_cmd(cmd.into(), callback, data)
        }
        unsafe extern "C" fn get_curr_row(editor: *mut crate::Editor) -> StringView<'static> {
            (*editor).row().as_slice().into()
        }
        unsafe extern "C" fn update_curr_row(editor: *mut crate::Editor, new_row: StringView) {
            let new_row: &[u8] = new_row.into();
            (*editor).log(format!("Updating current row: {:?}", new_row));
            let row = (*editor).row();
            row.clear();
            row.extend_from_slice(new_row);
        }
        Self {
            editor: editor as *mut _ as _,
            plugin,
            set_status,
            add_cmd,
            get_curr_row,
            update_curr_row,
        }
    }
}

pub struct Plugin {
    handle: *mut libc::c_void,
    pub init: extern "C" fn(*mut Api),
    pub cmds: Vec<(String, CmdCallback, *mut libc::c_void)>,
}

impl Plugin {
    pub fn load(path: String) -> Result<Self, CString> {
        let handle = unsafe { libc::dlopen(path.as_ptr() as _, libc::RTLD_NOW) };
        if handle.is_null() {
            let msg = unsafe {
                CString::from_raw(libc::dlerror())
            };
            return Err(msg);
        }

        let init = unsafe { libc::dlsym(handle, c"ers_plugin_init".as_ptr()) };
        if init.is_null() {
            let msg = unsafe {
                CString::from_raw(libc::dlerror())
            };
            return Err(msg);
        }

        Ok(Self { handle, init: unsafe  { std::mem::transmute(init) }, cmds: Vec::new() })
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
