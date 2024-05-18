
pub struct Plugin {
    pub init: extern "C" fn(*mut Api),
    pub cmds: Vec<(String, CmdCallback, *mut libc::c_void)>,
    pub on_render: Option<(
        extern "C" fn(*mut Api, *mut libc::c_void),
        *mut libc::c_void,
    )>,
}

impl Plugin {
    pub fn load(path: String) -> Result<Self, CString> {
        todo!()
    }

    pub fn add_cmd(&mut self, cmd: String, callback: CmdCallback, data: *mut libc::c_void) {
        self.cmds.push((cmd, callback, data));
    }
}

impl std::ops::Drop for Plugin {
    fn drop(&mut self) {
        todo!()
    }
}
