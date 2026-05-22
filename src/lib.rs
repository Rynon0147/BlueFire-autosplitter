#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod offsets;
use crate::offsets::get_offsets;
mod splitter_settings;
//use crate::offsets::get_offsets;

use asr::{
    future::{
        next_tick, 
        retry
    }, 
    settings::Gui,
    Process, 
    time::Duration,
    print_message, 
    watcher::Watcher, 
    game_engine::unreal::{
        Module, 
        Version
    },
    PointerSize::{
        Bit64},
    timer::{
        reset, 
        set_game_time, 
        set_variable, 
        set_variable_float, 
        set_variable_int, 
        split, 
        start, 
        state, 
        TimerState,
        pause_game_time,
    },
    Address64,
    string::ArrayWString,
};
use alloc::{
    string::String,
    format,
};

asr::async_main!(stable);
asr::panic_handler!();

const DEBUG: bool = false;

async fn main() {
    // Set up some general state and settings.
    let process_name: &str = "PROA34-Win64-Shipping.exe";
    let mut settings = splitter_settings::Settings::register();
    splitter_settings::set_tooltips();

    loop {
        // wait until process is found
        let process: Process = Process::wait_attach(process_name).await;
        print_message("Process found!");

        process.until_closes(async {
            // Load some initial information from the process.
            let module = retry(|| Module::attach(
                &process,
                Version::V4_25,
                process.get_module_address(process_name).unwrap(),
            )).await;

            let offsets = get_offsets();
            let mut split_states: [i32; 32] = [0; 32];

            // Game Timer
            let mut watch_total_centiseconds: Watcher<f32> = Watcher::new();
            watch_total_centiseconds.update_infallible(0f32);
            if DEBUG {set_variable_float("Centiseconds", 0f32);}

            // Current shrine
            let mut watch_current_shrine: Watcher<u32> = Watcher::new();
            if DEBUG {set_variable_int("Shrine", 0u32);}

            // Current event size
            let mut watch_current_event_size: Watcher<u32> = Watcher::new();
            watch_current_event_size.update_infallible(0u32);
            if DEBUG {set_variable_int("EventSize", 0u32);}

            // Current event
            let mut watch_current_event: Watcher<String> = Watcher::new();
            watch_current_event.update_infallible(String::from("NONE"));
            if DEBUG {set_variable("Event", "NONE");}


            print_message("Loop start");
            
            loop {
                settings.update();

                if DEBUG {set_variable_int("World", module.g_world().value());}

                // Game Timer
                if let Ok(time) = process.read_pointer_path::<f32>(
                    module.g_world(),
                    Bit64,
                    &offsets.centiseconds,
                ) {
                    if time > 0f32 {
                        watch_total_centiseconds.update_infallible(time);
                        if DEBUG {set_variable_float("Centiseconds", time);}
                        set_game_time(Duration::seconds_f32(time/100.0));
                    }
                }

                // Shrines
                if let Ok(flag) = process.read_pointer_path::<u32>(
                    module.g_world(),
                    Bit64,
                    &offsets.last_shrine,
                ) {
                    watch_current_shrine.update_infallible(flag);
                    if DEBUG {set_variable_int("Shrine", flag);}
                }


                // Events
                if let Ok(size) = process.read_pointer_path::<u32>(
                    module.g_world(), 
                    Bit64, 
                    &offsets.events_size
                ) {
                    //if DEBUG {set_variable_int("EventSize", size);}
                    watch_current_event_size.update_infallible(size);
                    if watch_current_event_size.pair.unwrap().changed() && size > 1 {

                        let offset = (size - 1) * 0x10;
                        //if DEBUG {set_variable_int("Event offset" ,offset);}
                        if let Ok(event) = process.read_pointer_path::<u32>(
                            module.g_world(), 
                            Bit64, 
                            &offsets.events_array,
                        ) {
                            //if DEBUG {set_variable_int("Events", event);}
                            let black_magic = event + offset;
                            //if DEBUG {set_variable_int("black magic", black_magic);}
                            let magic_address = Address64::new(black_magic.into());
                            if let Ok(event_name_address) = process.read_pointer(magic_address, Bit64){
                                //if DEBUG {set_variable_int("Event string address", event_name_address.value());}
                                if let Ok(event_string) = process.read::<ArrayWString<255>>(event_name_address){
                                    let plsmanijustwantastr: String = String::from_utf16_lossy(&event_string);
                                    //if DEBUG {set_variable("Event", str_event);}
                                    watch_current_event.update_infallible(plsmanijustwantastr);
                                }
                            }
                        }
                    }

                }

                if let Ok(chunk) = process.read_pointer_path::<u32>(
                                            module.g_world(),
                                            Bit64,
                                            &offsets.streaming_chunk,
                                        ) 
                {
                    if DEBUG {set_variable_int("chunk", chunk);}
                }
                // Cutscene (dont need yet~)
                
                if let Ok(flag) = process.read_pointer_path::<bool>(
                    module.g_world(),
                    Bit64,
                    &offsets.cutscene,
                ) {
                    if flag{
                        if DEBUG {set_variable("cutscene", "yay");}
                    } else {
                        if DEBUG {set_variable("cutscene", "buh");}
                    }
                }
                
                
                //completion
                if settings.show_completion {
                    if let Ok(flag) = process.read_pointer_path::<u32>(
                        module.g_world(),
                        Bit64,
                        &offsets.completion,
                    ) {
                        let form_string = format!("{}.{:02}%", flag / 100, flag % 100);
                        set_variable("Completion", form_string.as_str());
                    }
                }

                match state(){
                    TimerState::NotRunning => {
                        //start timer
                        if let Some(current_event_string) = &watch_current_event.pair {
                            if current_event_string.changed() && current_event_string.current.as_str() == "IntroScene" {
                                reset_all(&mut split_states);
                            }
                        }
                    }
                    
                    TimerState::Paused | TimerState::Running => {
                        if let Some(current_shrine) = watch_current_shrine.pair {
                            if current_shrine.changed(){
                                //print_message("new shrine");
                                match current_shrine.current{
                                    4 => {
                                        split_setting_check(FK_SHRINE, settings.fire_keep_fire_shrine, &mut split_states);
                                    }
                                    3 => {
                                        split_setting_check(AT_SHRINE, settings.arcane_tunnels_fire_shrine, &mut split_states);
                                    }
                                    0 => {
                                        split_setting_check(SHC_SHRINE, settings.stoneheart_city_fire_shrine, &mut split_states);
                                    }
                                    1 => {
                                        split_setting_check(AP_SHRINE, settings.abandoned_path_fire_shrine, &mut split_states);
                                    }
                                    2 => {
                                        split_setting_check(TG_SHRINE, settings.temple_gardens_fire_shrine, &mut split_states);
                                    }
                                    5 => {
                                        split_setting_check(FFR_SHRINE, settings.firefall_river_fire_shrine, &mut split_states);
                                    }
                                    6 => {
                                        split_setting_check(SH_SHRINE, settings.steam_house_fire_shrine, &mut split_states);
                                    }
                                    //DLC
                                    8 => {
                                        split_setting_check(VG_SHRINE, settings.void_gate_fire_shrine, &mut split_states);
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if let Some(current_event_string) = &watch_current_event.pair {
                            if current_event_string.changed() {
                                //do the event once
                                let one_time_event = current_event_string.current.clone();
                                watch_current_event.update_infallible(one_time_event.clone());
                                //print_message(one_time_event.as_str());
                                match one_time_event.as_str() {
                                    //reset
                                    "IntroScene" => {
                                        if watch_total_centiseconds.pair.unwrap().current > 0f32{
                                            
                                            reset_all(&mut split_states);
                                        }
                                    }
                                    //vessels
                                    "BP_BeiraVesselBase_Graveyard" => {
                                        split_setting_check(AP_SOUL_FRAGMENTS_END, settings.ap_soul_fragments_end, &mut split_states);
                                    }
                                    "BP_BeiraVesselBase_TempleGardens" => {
                                        split_setting_check(TG_SOUL_FRAGMENTS_END, settings.tg_soul_fragments_end, &mut split_states);
                                    }
                                    "BP_BeiraVesselBase_LakeMolva" => {
                                        split_setting_check(FFR_SOUL_FRAGMENTS_END, settings.ffr_soul_fragments_end, &mut split_states);
                                    }
                                    "BP_BeiraVesselBase_GameIntro" =>{
                                        split_setting_check(FK_SOUL_FRAGMENTS_END, settings.fk_soul_fragments_end, &mut split_states);
                                    }
                                    //bosses
                                    "NuosTempleEndCutscene" => {
                                        split_setting_check(GRUH, settings.gruh_dead, &mut split_states);
                                    }
                                    "UthasEndCutscene" => {
                                        split_setting_check(CROH, settings.croh_dead, &mut split_states);
                                    }
                                    "TeleportTemple" => {
                                        // Streaming Chunk
                                        if let Ok(chunk) = process.read_pointer_path::<u32>(
                                            module.g_world(),
                                            Bit64,
                                            &offsets.streaming_chunk,
                                        ) {
                                            if DEBUG {set_variable_int("Chunk", chunk);}
                                            match chunk {
                                                14u32 =>   {
                                                    split_setting_check(SIRION, settings.sirion_dead, &mut split_states);
                                                }
                                                2u32 =>{
                                                    split_setting_check(BEIRA, settings.beira_dead, &mut split_states);
                                                }
                                                9u32 => {
                                                    split_setting_check(SAMAEL, settings.samael_dead, &mut split_states);
                                                }
                                                _ => {}                                     
                                            }
                                        }
                                    }
                                    "BossQueen" => {
                                        split_setting_check(QUEEN, settings.queen_dead, &mut split_states);
                                    }
                                    //abilities
                                    "Chest_A01_Keep_Shield" => {
                                        split_setting_check(SHIELD, settings.shield, &mut split_states);
                                    }
                                    "IDTutorial_FireBall" => {
                                        split_setting_check(FIREBALL, settings.fireball, &mut split_states);
                                    }
                                    "IDTutorial_Spin Attack" => {
                                        split_setting_check(SPIN, settings.spin, &mut split_states);
                                    }
                                    "IDTutorial_Wall Run" => {
                                        split_setting_check(WALLRUN, settings.wall_run, &mut split_states);
                                    }
                                    "IDTutorial_Double Jump" => {
                                        split_setting_check(DOUBLEJUMP, settings.double_jump, &mut split_states);
                                    }
                                    "IDTutorial_Warp" => {
                                        split_setting_check(FASTTRAVEL, settings.fast_travel, &mut split_states);
                                    }
                                    /*
                                    DLC
                                     */
                                    //Von
                                    /*
                                    "NAME" => {
                                        split_setting_check(VOID_DLC_X_X_X, settings.fast_travel, &mut split_states);
                                    }
                                    */
                                    //Voids
                                    //Blue start
                                    /*
                                    "MyrasRise" => {
                                        split_setting_check(VOID_DLC_B_S_1, settings.void_dlc_b_s_1, &mut split_states);
                                    }
                                    "AmericsKnowledge" => {
                                        split_setting_check(VOID_DLC_B_S_2, settings.void_dlc_b_s_2, &mut split_states);
                                    }
                                    "TheWiseHineoto" => {
                                        split_setting_check(VOID_DLC_B_S_3, settings.void_dlc_b_s_3, &mut split_states);
                                    }
                                    "GordurTheShifter" => {
                                        split_setting_check(VOID_DLC_B_S_4, settings.void_dlc_b_s_4, &mut split_states);
                                    }
                                    "ToricsOdyssey" => {
                                        split_setting_check(VOID_DLC_B_S_5, settings.void_dlc_b_s_5, &mut split_states);
                                    }
                                    "AscendanceOfKaidens" => {
                                        split_setting_check(VOID_DLC_B_S_6, settings.void_dlc_b_s_6, &mut split_states);
                                    }
                                    "TheasTower" => {
                                        split_setting_check(VOID_DLC_B_S_7, settings.void_dlc_b_s_7, &mut split_states);
                                    }
                                    "SaronTheBetrayer" => {
                                        split_setting_check(VOID_DLC_B_S_8, settings.void_dlc_b_s_8, &mut split_states);
                                    }
                                    //Blue end
                                    "GodStone_32_MyrasRise" => {
                                        split_setting_check(VOID_DLC_B_E_1, settings.void_dlc_b_e_1, &mut split_states);
                                    }
                                    "GodStone_20_AmericsKnowledge" => {
                                        split_setting_check(VOID_DLC_B_E_2, settings.void_dlc_b_e_2, &mut split_states);
                                    }
                                    "GodStone_28_TheWiseHineoto" => {
                                        split_setting_check(VOID_DLC_B_E_3, settings.void_dlc_b_e_3, &mut split_states);
                                    }
                                    "GodStone_23_GordurTheShifter" => {
                                        split_setting_check(VOID_DLC_B_E_4, settings.void_dlc_b_e_4, &mut split_states);
                                    }
                                    "GodStone_33_ToricsOdyssey" => {
                                        split_setting_check(VOID_DLC_B_E_5, settings.void_dlc_b_e_5, &mut split_states);
                                    }
                                    "GodStone_30_AscendanceOfKaidens" => {
                                        split_setting_check(VOID_DLC_B_E_6, settings.void_dlc_b_e_6, &mut split_states);
                                    }
                                    "GodStone_18_TheasTower" => {
                                        split_setting_check(VOID_DLC_B_E_7, settings.void_dlc_b_e_7, &mut split_states);
                                    }
                                    "GodStone_29_SaronTheBetrayer" => {
                                        split_setting_check(VOID_DLC_B_E_8, settings.void_dlc_b_e_8, &mut split_states,);
                                    }

                                    //Red start
                                    "RowesTemple" => {
                                        split_setting_check(VOID_DLC_R_S_1, settings.void_dlc_r_s_1, &mut split_states);
                                    }
                                    "TolonsRace" => {
                                        split_setting_check(VOID_DLC_R_S_2, settings.void_dlc_r_s_2, &mut split_states);
                                    }
                                    "AdrehLegacy" => {
                                        split_setting_check(VOID_DLC_R_S_3, settings.void_dlc_r_s_3, &mut split_states);
                                    }
                                    "AscendanceOfKinau" => {
                                        split_setting_check(VOID_DLC_R_S_4, settings.void_dlc_r_s_4, &mut split_states);
                                    }
                                    "BalanceOfSoun" => {
                                        split_setting_check(VOID_DLC_R_S_5, settings.void_dlc_r_s_5, &mut split_states);
                                    }
                                    "PainOfKimeo" => {
                                        split_setting_check(VOID_DLC_R_S_6, settings.void_dlc_r_s_6, &mut split_states);
                                    }
                                    "LeapOfVoromir" => {
                                        split_setting_check(VOID_DLC_R_S_7, settings.void_dlc_r_s_7, &mut split_states);
                                    }
                                    "VarioTheInvisible" => {
                                        split_setting_check(VOID_DLC_R_S_8, settings.void_dlc_r_s_8, &mut split_states);
                                    }
                                    //Red end
                                    "GodStone_27_RowesTemple" => {
                                        split_setting_check(VOID_DLC_R_E_1, settings.void_dlc_r_e_1, &mut split_states);
                                    }
                                    "GodStone_24_TolonsRace" => {
                                        split_setting_check(VOID_DLC_R_E_2, settings.void_dlc_r_e_2, &mut split_states);
                                    }
                                    "GodStone_19_AdrehLegacy" => {
                                        split_setting_check(VOID_DLC_R_E_3, settings.void_dlc_r_e_3, &mut split_states);
                                    }
                                    "GodStone_22_AscendanceOfKinau" => {
                                        split_setting_check(VOID_DLC_R_E_4, settings.void_dlc_r_e_4, &mut split_states);
                                    }
                                    "GodStone_31_BalanceOfSoun" => {
                                        split_setting_check(VOID_DLC_R_E_5, settings.void_dlc_r_e_5, &mut split_states);
                                    }
                                    "GodStone_26_PainOfKimeo" => {
                                        split_setting_check(VOID_DLC_R_E_6, settings.void_dlc_r_e_6, &mut split_states);
                                    }
                                    "GodStone_25_LeapOfVoromir" => {
                                        split_setting_check(VOID_DLC_R_E_7, settings.void_dlc_r_e_7, &mut split_states);
                                    }
                                    "GodStone_21_VarioTheInvisible" => {
                                        split_setting_check(VOID_DLC_R_E_8, settings.void_dlc_r_e_8, &mut split_states);
                                    }
                                    */
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
                next_tick().await;
            }
            
        })
        .await;
    }
}


fn split_setting_check(index: usize, setting: bool, split_states: &mut [i32;32]){
    if setting && split_states[index] == 0{
        split();
        split_states[index] = 1;
    }

}


fn reset_all(split_states: &mut [i32;32]){
    split_states.fill(0);
 
    reset();
    start();
    pause_game_time();
    
}

//shrines
const SHC_SHRINE: usize = 0;
const AP_SHRINE: usize = 1;
const TG_SHRINE: usize = 2;
const AT_SHRINE: usize = 3;
const FK_SHRINE: usize = 4;
const FFR_SHRINE: usize = 5;
const SH_SHRINE: usize = 6;

//vesselsouls
const AP_SOUL_FRAGMENTS_END: usize = 7;
const TG_SOUL_FRAGMENTS_END: usize = 8;
const FFR_SOUL_FRAGMENTS_END: usize = 9;
const FK_SOUL_FRAGMENTS_END: usize = 10;

//bossesgit reset --hard
const GRUH: usize = 11;
const CROH: usize = 12;
const SIRION: usize = 13;
const BEIRA: usize = 14;
const SAMAEL: usize = 15;
const QUEEN: usize = 16;

//abilities
const SHIELD: usize = 17;
const FIREBALL: usize = 18;
const SPIN: usize = 19;
const WALLRUN: usize = 20;
const DOUBLEJUMP: usize = 21;
const FASTTRAVEL: usize = 22;

//DLC shrine
const VG_SHRINE: usize = 23;
/*
//DLC voids blue
const VOID_DLC_B_S_1: usize = 24;
const VOID_DLC_B_S_2: usize = 25;
const VOID_DLC_B_S_3: usize = 26;
const VOID_DLC_B_S_4: usize = 27;
const VOID_DLC_B_S_5: usize = 28;
const VOID_DLC_B_S_6: usize = 29;
const VOID_DLC_B_S_7: usize = 30;
const VOID_DLC_B_S_8: usize = 31;

//DLC voids red
const VOID_DLC_R_S_1: usize = 32;
const VOID_DLC_R_S_2: usize = 33;
const VOID_DLC_R_S_3: usize = 34;
const VOID_DLC_R_S_4: usize = 35;
const VOID_DLC_R_S_5: usize = 36;
const VOID_DLC_R_S_6: usize = 37;
const VOID_DLC_R_S_7: usize = 38;
const VOID_DLC_R_S_8: usize = 39;

//DLC voids blue end
const VOID_DLC_B_E_1: usize = 40;
const VOID_DLC_B_E_2: usize = 41;
const VOID_DLC_B_E_3: usize = 42;
const VOID_DLC_B_E_4: usize = 43;
const VOID_DLC_B_E_5: usize = 44;
const VOID_DLC_B_E_6: usize = 45;
const VOID_DLC_B_E_7: usize = 46;
const VOID_DLC_B_E_8: usize = 47;

//DLC voids red end
const VOID_DLC_R_E_1: usize = 48;
const VOID_DLC_R_E_2: usize = 49;
const VOID_DLC_R_E_3: usize = 50;
const VOID_DLC_R_E_4: usize = 51;
const VOID_DLC_R_E_5: usize = 52;
const VOID_DLC_R_E_6: usize = 53;
const VOID_DLC_R_E_7: usize = 54;
const VOID_DLC_R_E_8: usize = 55;

//DLC boss
const BOSS_DLC: usize = 56;
*/