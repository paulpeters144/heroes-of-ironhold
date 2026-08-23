#[derive(Clone, Copy)]
pub struct SaveSlot {
    pub name: &'static str,
    pub chapter: &'static str,
    pub play_time: &'static str,
    pub level: u8,
    pub timestamp: &'static str,
    pub progress: f32,
}

pub const SAVE_SLOTS: [SaveSlot; 0] = [
    // SaveSlot {
    //     name: "SLOT 1",
    //     chapter: "Ch. 3: Neon Foundry",
    //     play_time: "04:32:18",
    //     level: 14,
    //     timestamp: "2026-08-17 22:14",
    //     progress: 0.42,
    // },
    // SaveSlot {
    //     name: "SLOT 2",
    //     chapter: "Ch. 7: Hollow Spire",
    //     play_time: "18:07:44",
    //     level: 31,
    //     timestamp: "2026-08-18 09:03",
    //     progress: 0.76,
    // },
    // SaveSlot {
    //     name: "SLOT 3",
    //     chapter: "Ch. 1: Boot Sector",
    //     play_time: "00:41:02",
    //     level: 3,
    //     timestamp: "2026-08-12 20:57",
    //     progress: 0.08,
    // },
];
