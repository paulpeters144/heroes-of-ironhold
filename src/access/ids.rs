#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetKind {
    Texture,
    Image,
    Font,
    Sound,
    File,
    Shader,
}

pub trait AssetId {
    fn path(&self) -> String;
    fn kind(&self) -> AssetKind;
}

pub mod texture {
    use crate::access::ids::{AssetId, AssetKind};

    #[derive(Clone, Copy, Debug)]
    pub enum Texture {}

    impl AssetId for Texture {
        fn path(&self) -> String {
            match *self {}
        }

        fn kind(&self) -> AssetKind {
            match *self {}
        }
    }
}

pub mod font {
    use crate::access::ids::{AssetId, AssetKind};

    #[derive(Clone, Copy, Debug)]
    pub enum Font {
        Pixellari,
        Tiny04b03,
    }

    impl AssetId for Font {
        fn path(&self) -> String {
            match self {
                Self::Pixellari => "fonts/Pixellari.ttf".to_string(),
                Self::Tiny04b03 => "fonts/04b_03.ttf".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Font
        }
    }
}

pub mod sound {
    use crate::access::ids::{AssetId, AssetKind};

    #[derive(Clone, Copy, Debug)]
    pub enum Sound {}

    impl AssetId for Sound {
        fn path(&self) -> String {
            match *self {}
        }

        fn kind(&self) -> AssetKind {
            match *self {}
        }
    }
}

pub mod file {
    use crate::access::ids::{AssetId, AssetKind};

    #[derive(Clone, Copy, Debug)]
    pub enum File {
        TestTmx,
        HoiBgTsx,
        HoiCharsTsx,
    }

    impl AssetId for File {
        fn path(&self) -> String {
            match self {
                Self::TestTmx => "tiled/test-scene/test.tmx".to_string(),
                Self::HoiBgTsx => "tiled/test-scene/hoi-bg-pixelated.tsx".to_string(),
                Self::HoiCharsTsx => "tiled/test-scene/hoi-chars.tsx".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::File
        }
    }
}

pub mod shader {
    use crate::access::ids::{AssetId, AssetKind};

    #[derive(Clone, Copy, Debug)]
    pub enum Shader {
        PixelSnapVert,
        PixelSnapFrag,
        JitterFreeVert,
        JitterFreeFrag,
        DashFxVert,
        DashAfterimageFrag,
        FlashWhiteFrag,
        DemonDeathFrag,
        KnightGoldenGlowFrag,
        SpinDiscFrag,
        DivineStanceFrag,
    }

    impl AssetId for Shader {
        fn path(&self) -> String {
            match self {
                Self::PixelSnapVert => "shaders/pixel_snap.vert".to_string(),
                Self::PixelSnapFrag => "shaders/pixel_snap.frag".to_string(),
                Self::JitterFreeVert => "shaders/jitter_free.vert".to_string(),
                Self::JitterFreeFrag => "shaders/jitter_free.frag".to_string(),
                Self::DashFxVert => "shaders/dash_fx.vert".to_string(),
                Self::DashAfterimageFrag => "shaders/dash_afterimage.frag".to_string(),
                Self::FlashWhiteFrag => "shaders/flash_white.frag".to_string(),
                Self::DemonDeathFrag => "shaders/demon_death.frag".to_string(),
                Self::KnightGoldenGlowFrag => "shaders/knight_golden_glow.frag".to_string(),
                Self::SpinDiscFrag => "shaders/spin_disc.frag".to_string(),
                Self::DivineStanceFrag => "shaders/divine_stance.frag".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Shader
        }
    }
}

pub mod images {
    use crate::access::ids::{AssetId, AssetKind};

    #[derive(Clone, Copy, Debug)]
    pub enum Knight {
        Knight1,
        Knight2,
        Knight3,
        Shield1,
        Shield2,
        Shield3,
        Sword1,
        Sword2,
        Sword3,
        ThrustGraphic,
        SwipeGraphic,
        Face,
        Hit,
        SpinShield,
    }

    impl AssetId for Knight {
        fn path(&self) -> String {
            match self {
                Self::Knight1 => "images/knight/anim-knight-1.png".to_string(),
                Self::Knight2 => "images/knight/anim-knight-2.png".to_string(),
                Self::Knight3 => "images/knight/anim-knight-3.png".to_string(),
                Self::Shield1 => "images/knight/anim-knight-shield-1.png".to_string(),
                Self::Shield2 => "images/knight/anim-knight-shield-2.png".to_string(),
                Self::Shield3 => "images/knight/anim-knight-shield-3.png".to_string(),
                Self::Sword1 => "images/knight/anim-knight-sword-1.png".to_string(),
                Self::Sword2 => "images/knight/anim-knight-sword-2.png".to_string(),
                Self::Sword3 => "images/knight/anim-knight-sword-3.png".to_string(),
                Self::ThrustGraphic => "images/knight/static-knight-thrust-graphic.png".to_string(),
                Self::SwipeGraphic => "images/knight/static-knight-swipe-graphic.png".to_string(),
                Self::Face => "images/knight/static-knight-face.png".to_string(),
                Self::Hit => "images/knight/static-knight-hit.png".to_string(),
                Self::SpinShield => "images/knight/static-spin-shield.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum SkillIcon {
        BladeBarrage,
        Shield,
        Blast,
    }

    impl AssetId for SkillIcon {
        fn path(&self) -> String {
            match self {
                Self::BladeBarrage => "images/knight/icon-swords.png".to_string(),
                Self::Shield => "images/knight/icon-shield.png".to_string(),
                Self::Blast => "images/knight/icon-blast.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Ui {
        SkillSlot,
    }

    impl AssetId for Ui {
        fn path(&self) -> String {
            match self {
                Self::SkillSlot => "images/ui/skill-slot.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Enemy {
        RamHead,
        RamHeadHit,
    }

    impl AssetId for Enemy {
        fn path(&self) -> String {
            match self {
                Self::RamHead => "images/enemies/anim-ram-head.png".to_string(),
                Self::RamHeadHit => "images/enemies/static-ramhead-hit.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Megabot {
        Idle,
        Run,
        Jump,
        Hurt,
    }

    impl AssetId for Megabot {
        fn path(&self) -> String {
            match self {
                Self::Idle => "images/megabot/anim-mb-idle.png".to_string(),
                Self::Run => "images/megabot/anim-mb-run.png".to_string(),
                Self::Jump => "images/megabot/anim-mb-jump.png".to_string(),
                Self::Hurt => "images/megabot/anim-mb-hurt.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Pointer;

    impl AssetId for Pointer {
        fn path(&self) -> String {
            "images/pointer.png".to_string()
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct TileSet;

    impl AssetId for TileSet {
        fn path(&self) -> String {
            "images/tile-set.png".to_string()
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum TilesetImage {
        HoiBg,
        HoiChars,
    }

    impl AssetId for TilesetImage {
        fn path(&self) -> String {
            match self {
                Self::HoiBg => "tiled/test-scene/hoi-bg-pixelated.png".to_string(),
                Self::HoiChars => "tiled/test-scene/hoi-chars.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Image
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum Scene {
        MenuBackground,
        Scroll,
        TitleText,
        LargeRamhead,
        RamheadWiz,
        FireDemon1,
        FireDemon2,
        Ramhead,
        AssassinDemon,
        Bird,
        BerserkIdle,
        KnightIdle,
        WizardIdle,
        AssassinIdle,
        ArcherIdle,
        Flag,
        Torch1,
        Torch2,
    }

    impl AssetId for Scene {
        fn path(&self) -> String {
            match self {
                Self::MenuBackground => "images/scenes/menu/menu-background.png".to_string(),
                Self::Scroll => "images/scenes/menu/scroll.png".to_string(),
                Self::TitleText => "images/scenes/menu/title-text.png".to_string(),
                Self::LargeRamhead => "images/scenes/menu/anim-large-ramhead.png".to_string(),
                Self::RamheadWiz => "images/scenes/menu/anim-ramhead-wiz.png".to_string(),
                Self::FireDemon1 => "images/scenes/menu/anim-fire-demon-1.png".to_string(),
                Self::FireDemon2 => "images/scenes/menu/anim-fire-demon-2.png".to_string(),
                Self::Ramhead => "images/scenes/menu/anim-ramhead.png".to_string(),
                Self::AssassinDemon => "images/scenes/menu/anim-assassin-demon.png".to_string(),
                Self::Bird => "images/scenes/menu/anim-bird.png".to_string(),
                Self::BerserkIdle => "images/scenes/menu/static-hero-berserk.png".to_string(),
                Self::KnightIdle => "images/scenes/menu/static-hero-knight.png".to_string(),
                Self::WizardIdle => "images/scenes/menu/static-hero-wizard.png".to_string(),
                Self::AssassinIdle => "images/scenes/menu/static-hero-assassin.png".to_string(),
                Self::ArcherIdle => "images/scenes/menu/static-hero-archer.png".to_string(),
                Self::Flag => "images/scenes/menu/anim-flag.png".to_string(),
                Self::Torch1 => "images/scenes/menu/anim-torch-1.png".to_string(),
                Self::Torch2 => "images/scenes/menu/anim-torch-2.png".to_string(),
            }
        }

        fn kind(&self) -> AssetKind {
            AssetKind::Texture
        }
    }
}
