
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
            init: unsafe { std::mem::transmute::<*mut libc::c_void, extern "C" fn(*mut Api)>(init) },
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

