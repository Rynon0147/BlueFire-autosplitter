use asr::settings::gui::{set_tooltip, Title};
use asr::settings::Gui;


pub fn set_tooltips() {
    {
        set_tooltip(
            "show_completion",
            r##"Sets the variable "Completion". Can be displayed by LiveSplit as text."##,
        );
    }
}

#[derive(Gui)]
pub struct Settings {
    #[heading_level = 0]
    fire_shrines: Title,
    #[default = false]
    pub fire_keep_fire_shrine: bool,
    #[default = false]
    pub arcane_tunnels_fire_shrine: bool,
    #[default = false]
    pub stoneheart_city_fire_shrine: bool,
    #[default = false]
    pub abandoned_path_fire_shrine: bool,
    #[default = false]
    pub temple_gardens_fire_shrine: bool,
    #[default = false]
    pub firefall_river_fire_shrine: bool,
    #[default = false]
    pub steam_house_fire_shrine: bool,

    #[heading_level = 0]
    soul_fragments: Title,
    #[default = false]
    pub ap_soul_fragments_end: bool,
    #[default = false]
    pub tg_soul_fragments_end: bool,
    #[default = false]
    pub ffr_soul_fragments_end: bool,
    #[default = false]
    pub fk_soul_fragments_end: bool,

    #[heading_level = 0]
    bosses: Title,
    #[default = false]
    pub gruh_dead: bool,
    #[default = false]
    pub croh_dead: bool,
    #[default = false]
    pub sirion_dead: bool,
    #[default = false]
    pub beira_dead: bool,
    #[default = false]
    pub samael_dead: bool,
    #[default = true]
    pub queen_dead: bool,

    #[heading_level = 0]
    abilities: Title,
    #[default = false]
    pub shield: bool,
    #[default = false]
    pub fireball: bool,
    #[default = false]
    pub spin: bool,
    #[default = false]
    pub wall_run: bool,
    #[default = false]
    pub double_jump: bool,
    #[default = false]
    pub fast_travel: bool,

    #[heading_level = 0]
    other_settings: Title,
    #[default = true]
    pub show_completion: bool,
}