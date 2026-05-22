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
    blue_fire: Title,
    #[heading_level = 1]
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

    #[heading_level = 1]
    soul_fragments: Title,
    #[default = false]
    pub ap_soul_fragments_end: bool,
    #[default = false]
    pub tg_soul_fragments_end: bool,
    #[default = false]
    pub ffr_soul_fragments_end: bool,
    #[default = false]
    pub fk_soul_fragments_end: bool,

    #[heading_level = 1]
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

    #[heading_level = 1]
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

    #[heading_level = 1]
    transitions: Title,
    #[default = false]
    ///Enter Forest Shrine from Stoneheart City
    pub transition_shc_to_forest: bool,
    ///Enter Uthas Temple from Abandoned Path
    #[default = false]
    pub transition_ap_to_uthas: bool,

    #[heading_level = 1]
    spirits: Title,
    #[default = false]
    ///Fire Keep Tear
    pub spirit_fire_keep_tear: bool,
    ///Aerial Rat
    #[default = false]
    pub spirit_aerial_rat: bool,

    #[heading_level = 0]
    ///DLC
    dlc: Title,
    #[heading_level = 1]
    ///Shrines
    dlc_shrine: Title,
    #[default = false]
    pub void_gate_fire_shrine: bool,
/*
    #[heading_level = 1]
    ///Boss
    dlc_boss: Title,
    #[default = false]
    pub von: bool,

    
    #[heading_level = 1]
    ///voids
    dlc_voids: Title,
    #[heading_level = 2]
    ///Blue voids
    dlc_voids_blue: Title,
    #[heading_level = 3]
    ///Blue voids Start
    dlc_voids_blue_start: Title,
    #[default = false]
    ///Myra's Rise
    pub void_dlc_b_s_1: bool,
    #[default = false]
    ///Americ's Knowledge
    pub void_dlc_b_s_2: bool,
    #[default = false]
    ///The Wise Hineoto
    pub void_dlc_b_s_3: bool,
    #[default = false]
    ///Gordur The Shifter
    pub void_dlc_b_s_4: bool,
    #[default = false]
    ///Toric's Odyssey
    pub void_dlc_b_s_5: bool,
    #[default = false]
    ///Kaidens Ascension
    pub void_dlc_b_s_6: bool,
    #[default = false]
    ///Thea's Tower
    pub void_dlc_b_s_7: bool,
    #[default = false]
    ///Saron The Betrayer
    pub void_dlc_b_s_8: bool,

    #[heading_level = 3]
    ///Blue voids End
    dlc_voids_blue_end: Title,
    #[default = false]
    ///Myra's Rise
    pub void_dlc_b_e_1: bool,
    #[default = false]
    ///Americ's Knowledge
    pub void_dlc_b_e_2: bool,
    #[default = false]
    ///The Wise Hineoto
    pub void_dlc_b_e_3: bool,
    #[default = false]
    ///Gordur The Shifter
    pub void_dlc_b_e_4: bool,
    #[default = false]
    ///Toric's Odyssey
    pub void_dlc_b_e_5: bool,
    #[default = false]
    ///Kaidens Ascension
    pub void_dlc_b_e_6: bool,
    #[default = false]
    ///Thea's Tower
    pub void_dlc_b_e_7: bool,
    #[default = false]
    ///Saron the Betrayer
    pub void_dlc_b_e_8: bool,
    
    #[heading_level = 2]
    ///Red voids
    dlc_voids_red: Title,
    #[heading_level = 3]
    ///Red voids Start
    dlc_voids_red_start: Title,
    #[default = false]
    ///Rowe's Temple
    pub void_dlc_r_s_1: bool,
    #[default = false]
    ///Tolon's Race
    pub void_dlc_r_s_2: bool,
    #[default = false]
    ///Adreh's Legacy
    pub void_dlc_r_s_3: bool,
    #[default = false]
    ///Ascendance of Kinau
    pub void_dlc_r_s_4: bool,
    #[default = false]
    ///Balance of Soun
    pub void_dlc_r_s_5: bool,
    #[default = false]
    ///Pain of Kimeo
    pub void_dlc_r_s_6: bool,
    #[default = false]
    ///Leap of Voromir
    pub void_dlc_r_s_7: bool,
    #[default = false]
    ///Vario The Invisible
    pub void_dlc_r_s_8: bool,

    #[heading_level = 3]
    ///Red voids End
    dlc_voids_red_end: Title,
    #[default = false]
    ///Rowe's Temple
    pub void_dlc_r_e_1: bool,
    #[default = false]
    ///Tolon's Race
    pub void_dlc_r_e_2: bool,
    #[default = false]
    ///Adreh's Legacy
    pub void_dlc_r_e_3: bool,
    #[default = false]
    ///Ascendance of Kinau
    pub void_dlc_r_e_4: bool,
    #[default = false]
    ///Balance of Soun
    pub void_dlc_r_e_5: bool,
    #[default = false]
    ///Pain of Kimeo
    pub void_dlc_r_e_6: bool,
    #[default = false]
    ///Leap of Voromir
    pub void_dlc_r_e_7: bool,
    #[default = false]
    ///Vario The Invisible
    pub void_dlc_r_e_8: bool,
 */

    #[heading_level = 0]
    other_settings: Title,
    #[default = true]
    pub show_completion: bool,
}