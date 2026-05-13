pub fn get_offsets() -> Offsets {
    Offsets{
        centiseconds: [0, 0x30, 0xE8, 0x258, 0x10B8, 0x260],
        last_shrine: [0, 0x188, 0x351],
        cutscene: [0, 0x30, 0xE8, 0x258, 0xe70],
        events_array: [0, 0x188, 0x300 + 0x8],
        events_size: [0, 0x188, 0x300 + 0x8 + 0x8],
        completion: [0, 0x188, 0x338],
        streaming_chunk: [0, 0x30, 0xE8, 0x288, 0x700],
    }
}

pub(crate) struct Offsets {
    pub centiseconds: [u64; 6],
    pub last_shrine: [u64; 3],
    pub cutscene: [u64; 5],
    pub events_array: [u64; 3],
    pub events_size: [u64; 3],
    pub completion: [u64; 3],
    pub streaming_chunk: [u64; 5],
}