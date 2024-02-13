#[repr(C)]
pub(crate) struct BaseEvent {
    pub event_class: u32,
    pub event_type: u32,
    pub event_size: u32,
}

#[repr(C)]
pub(crate) struct FileEvent {
    pub base: BaseEvent,
    pub path_len: u32,
    pub path: u8,
}
