pub trait AssetId: Copy {
    fn id(&self) -> String;
}

pub mod texture {
    use crate::access::ids::AssetId;

    #[derive(Clone, Copy, Debug)]
    pub enum Id {}

    impl AssetId for Id {
        fn id(&self) -> String {
            match *self {}
        }
    }

    pub const TEXTURES: &[(Id, &str)] = &[];
}

pub mod font {
    use crate::access::ids::AssetId;

    #[derive(Clone, Copy, Debug)]
    pub enum Id {
        Pixellari,
        Tiny04b03,
    }

    impl AssetId for Id {
        fn id(&self) -> String {
            match self {
                Self::Pixellari => "pixellari".to_string(),
                Self::Tiny04b03 => "04b_03".to_string(),
            }
        }
    }

    pub const FONTS: &[(Id, &str)] = &[
        (Id::Pixellari, "fonts/Pixellari.ttf"),
        (Id::Tiny04b03, "fonts/04b_03.ttf"),
    ];
}

pub mod sound {
    use crate::access::ids::AssetId;

    #[derive(Clone, Copy, Debug)]
    pub enum Id {
        Blaster,
        Jump,
    }

    impl AssetId for Id {
        fn id(&self) -> String {
            match self {
                Self::Blaster => "blaster".to_string(),
                Self::Jump => "jump".to_string(),
            }
        }
    }

    pub const SOUNDS: &[(Id, &str)] = &[
        (Id::Blaster, "audio/blaster.wav"),
        (Id::Jump, "audio/jump.wav"),
    ];
}

pub mod file {
    use crate::access::ids::AssetId;

    #[derive(Clone, Copy, Debug)]
    pub enum Id {
        Scene1Tmx,
        TileSetTsx,
    }

    impl AssetId for Id {
        fn id(&self) -> String {
            match self {
                Self::Scene1Tmx => "scene_1_tmx".to_string(),
                Self::TileSetTsx => "tile_set_tsx".to_string(),
            }
        }
    }

    pub const FILES: &[(Id, &str)] = &[
        (Id::Scene1Tmx, "tiled/scene-1.tmx"),
        (Id::TileSetTsx, "tiled/tile-set.tsx"),
    ];
}

pub mod shader {
    use crate::access::ids::AssetId;

    #[derive(Clone, Copy, Debug)]
    pub enum Id {
        PixelSnapVert,
        PixelSnapFrag,
        JitterFreeVert,
        JitterFreeFrag,
    }

    impl AssetId for Id {
        fn id(&self) -> String {
            match self {
                Self::PixelSnapVert => "pixel_snap_vert".to_string(),
                Self::PixelSnapFrag => "pixel_snap_frag".to_string(),
                Self::JitterFreeVert => "jitter_free_vert".to_string(),
                Self::JitterFreeFrag => "jitter_free_frag".to_string(),
            }
        }
    }

    pub const SHADERS: &[(Id, &str)] = &[
        (Id::PixelSnapVert, "shaders/pixel_snap.vert"),
        (Id::PixelSnapFrag, "shaders/pixel_snap.frag"),
        (Id::JitterFreeVert, "shaders/jitter_free.vert"),
        (Id::JitterFreeFrag, "shaders/jitter_free.frag"),
    ];
}

pub mod image {
    use crate::access::ids::AssetId;

    #[derive(Clone, Copy, Debug)]
    pub enum Id {
        TileSet,
        MbRun,
        MbIdle,
        MbJump,
        MbHurt,
        E3,
    }

    impl AssetId for Id {
        fn id(&self) -> String {
            match self {
                Self::TileSet => "tile-set.png".to_string(),
                Self::MbRun => "anim-mb-run.png".to_string(),
                Self::MbIdle => "anim-mb-idle.png".to_string(),
                Self::MbJump => "anim-mb-jump.png".to_string(),
                Self::MbHurt => "anim-mb-hurt.png".to_string(),
                Self::E3 => "e-3.png".to_string(),
            }
        }
    }

    pub const IMAGES: &[(Id, &str)] = &[
        (Id::TileSet, "images/tile-set.png"),
        (Id::MbRun, "images/megabot/anim-mb-run.png"),
        (Id::MbIdle, "images/megabot/anim-mb-idle.png"),
        (Id::MbJump, "images/megabot/anim-mb-jump.png"),
        (Id::MbHurt, "images/megabot/anim-mb-hurt.png"),
        (Id::E3, "images/enemy/e-3.png"),
    ];

    pub mod hero {
        pub mod scene_one {
            use crate::access::ids::AssetId;

            #[derive(Clone, Copy, Debug)]
            pub enum Id {
                WalkAnim,
                IdleAnim,
            }

            impl AssetId for Id {
                fn id(&self) -> String {
                    match self {
                        Self::WalkAnim => "hero.scene_one.walk_anim".to_string(),
                        Self::IdleAnim => "hero.scene_one.idle_anim".to_string(),
                    }
                }
            }

            pub const FILES: &[(Id, &str)] = &[];
        }
    }
}
