#![no_std]
extern crate alloc;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;

mod offsets;

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
    //deep_pointer::DeepPointer,
    game_engine::unreal::{
//        FNameKey, 
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
//    string::ArrayCString,
    Address64,
    string::ArrayWString,
};
use alloc::{string::String, vec::Vec, string::ToString};
use crate::offsets::get_offsets;
use asr::settings::gui::Title;

asr::async_main!(stable);
asr::panic_handler!();

#[derive(Gui)]
struct Settings {
    fire_shrines: Title,
    fire_keep_fire_shrine: bool,
    arcane_tunnels_fire_shrine: bool,
    stoneheart_city_fire_shrine: bool,
    abandoned_path_fire_shrine: bool,
    temple_gardens_fire_shrine: bool,
    firefall_river_fire_shrine: bool,
    steam_house_fire_shrine: bool,

    soul_fragments: Title,
    ap_soul_fragments_end: bool,
    tg_soul_fragments_end: bool,
    ffr_soul_fragments_end: bool,
    fk_soul_fragments_end: bool,

    bosses: Title,
    gruh: bool,
    croh: bool,
    sirion: bool,
    beira: bool,
    samael: bool,
    queen: bool,
}
/*
struct Shrines {
    fire_keep_fire_shrine: bool,
    arcane_tunnels_fire_shrine: bool,
    stoneheart_city_fire_shrine: bool,
    abandoned_path_fire_shrine: bool,
    temple_gardens_fire_shrine: bool,
    firefall_river_fire_shrine: bool,
    steam_house_fire_shrine: bool,
}
*/

async fn main() {
    // Set up some general state and settings.
    let process_name: &str = "PROA34-Win64-Shipping.exe";
    let mut settings = Settings::register();

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

            let mut fresh_values: bool = false;
            let offsets = get_offsets();
            let mut split_states: [i32; 32] = [0; 32];

            // Game Timer
            let mut watch_total_centiseconds: Watcher<f32> = Watcher::new();
            watch_total_centiseconds.update_infallible(0f32);
            set_variable_float("Centiseconds", 0f32);

            // Current shrine
            let mut watch_current_shrine: Watcher<u32> = Watcher::new();
            set_variable_int("Shrine", 0u32);

            // Current event size
            let mut watch_current_event_size: Watcher<u32> = Watcher::new();
            watch_current_event_size.update_infallible(0u32);
            set_variable_int("EventSize", 0u32);

            // Current event
            let mut watch_current_event: Watcher<String> = Watcher::new();
            watch_current_event.update_infallible("NONE".to_string());
            set_variable("Event", &watch_current_event.pair.unwrap().current);


            print_message("Loop start");
            
            loop {
                settings.update();

                set_variable_int("World", module.g_world().value());
                
                // Game Timer
                if let Ok(time) = process.read_pointer_path::<f32>(
                    module.g_world(),
                    Bit64,
                    &offsets.centiseconds,
                ) {
                    if time > 0f32 {
                        watch_total_centiseconds.update_infallible(time);
                        set_variable_float("Centiseconds", time);
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
                    set_variable_int("Shrine", flag);
                }


                // Events
                if let Ok(size) = process.read_pointer_path::<u32>(
                    module.g_world(), 
                    Bit64, 
                    &offsets.events_size
                ) {
                    set_variable_int("EventSize", size);
                    watch_current_event_size.update_infallible(size);
                    if watch_current_event_size.pair.unwrap().changed() && size > 1 {

                        let offset = (size - 1) * 0x10;
                        set_variable_int("Event offset" ,offset);
                        if let Ok(event) = process.read_pointer_path::<u32>(
                            module.g_world(), 
                            Bit64, 
                            &offsets.events_array,
                        ) {
                            set_variable_int("Events", event);
                            let black_magic = event + offset;
                            set_variable_int("black magic", black_magic);
                            let magic_address = Address64::new(black_magic.into());
                            if let Ok(event_name_address) = process.read_pointer(magic_address, Bit64){
                                set_variable_int("Event string address", event_name_address.value());
                                if let Ok(event_string) = process.read::<ArrayWString<255>>(event_name_address){
                                    let plsmanijustwantastr: String = String::from_utf16_lossy(&event_string);
                                    let str_event: &str = plsmanijustwantastr.as_str();
                                    set_variable("Event", str_event);
                                }
                            }
                        }
                    }

                }
                
                // Cutscene (dont need yet~)
                if let Ok(flag) = process.read_pointer_path::<bool>(
                    module.g_world(),
                    Bit64,
                    &offsets.cutscene,
                ) {
                    if flag{
                        set_variable("cutscene", "yay");
                    } else {
                        set_variable("cutscene", "buh");
                    }
                }
                
                
                match state(){
                    TimerState::NotRunning => {
                        if !fresh_values{
                            reset_vars(&mut split_states);
                            fresh_values = true;
                        }
                        
                        if let Some(current_game_time) = watch_total_centiseconds.pair {
                            if current_game_time.old != current_game_time.current{
                                if current_game_time.current > 10f32{
                                    start();
                                    pause_game_time();
                                }
                            }
                        }
                    }
                    
                    TimerState::Paused | TimerState::Running => {
                        if fresh_values {
                            fresh_values = false;
                        }
                        if let Some(current_game_time) = watch_total_centiseconds.pair {
                            if current_game_time.current < current_game_time.old 
                            && current_game_time.current > 0f32 
                            && current_game_time.current < 5f32{
                                reset();
                            }
                        }

                        if let Some(current_shrine) = watch_current_shrine.pair {
                            if current_shrine.changed(){
                                print_message("new shrine");
                                match current_shrine.current{
                                    4 => {
                                        split_setting_check(FK_SHRINE, settings.fire_keep_fire_shrine, &mut split_states, "FK shrine");
                                    }
                                    3 => {
                                        split_setting_check(AT_SHRINE, settings.arcane_tunnels_fire_shrine, &mut split_states, "AT shrine");
                                    }
                                    0 => {
                                        split_setting_check(SHC_SHRINE, settings.stoneheart_city_fire_shrine, &mut split_states, "SHC shrine");
                                    }
                                    1 => {
                                        split_setting_check(AP_SHRINE, settings.abandoned_path_fire_shrine, &mut split_states, "AP shrine");
                                    }
                                    2 => {
                                        split_setting_check(TG_SHRINE, settings.temple_gardens_fire_shrine, &mut split_states, "TG shrine");
                                    }
                                    5 => {
                                        split_setting_check(FFR_SHRINE, settings.firefall_river_fire_shrine, &mut split_states, "FFR shrine");
                                    }
                                    6 => {
                                        split_setting_check(SH_SHRINE, settings.steam_house_fire_shrine, &mut split_states, "SH shrine");
                                    }
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

fn split_setting_check(index: usize, setting: bool, split_states: &mut [i32;32], var_string: &str){
    if setting && split_states[index] == 0{
        print_message("yesnt");
        split();
        split_states[index] = 1;
        set_variable_int(var_string, split_states[index]);
    }

}


fn reset_vars(split_states: &mut [i32;32]){
    split_states.fill(0);
    set_variable_int("FK shrine", 0);
    set_variable_int("AT shrine", 0);
    set_variable_int("SHC shrine", 0);
    set_variable_int("AP shrine", 0);
    set_variable_int("TG shrine", 0);
    set_variable_int("FFR shrine", 0);
    set_variable_int("SH shrine", 0);
}
/*

fn bool_split(){
    if !touched_shrines.steam_house_fire_shrine {
                                    split();
                                    touched_shrines.steam_house_fire_shrine = true;
    }
    if settings.fire_keep_fire_shrine
        && split_states[BAILEY_KEY] == 0
    {
        if let Some(bailey_key) = watch_bailey_key.pair {
            if bailey_key.changed_to(&true) {
                print_message("Split: Bailey Key Pickup");
                split_states[BAILEY_KEY] = 1;
                split()
            }
        }
    }
    match flag{
        4 => {
            if !touched_shrines.fire_keep_fire_shrine {
                split();
                touched_shrines.fire_keep_fire_shrine = true;
            }
        }
        3 => {
            if !touched_shrines.arcane_tunnels_fire_shrine {
                split();
                touched_shrines.arcane_tunnels_fire_shrine = true;
            }
        }
        0 => {
            if !touched_shrines.stoneheart_city_fire_shrine {
                split();
                touched_shrines.stoneheart_city_fire_shrine = true;
            }
        }
        1 => {
            if !touched_shrines.abandoned_path_fire_shrine {
                split();
                touched_shrines.abandoned_path_fire_shrine = true;
            }
        }
        2 => {if !touched_shrines.temple_gardens_fire_shrine {
                split();
                touched_shrines.temple_gardens_fire_shrine = true;
            }
        }
        5 => {if !touched_shrines.firefall_river_fire_shrine {
                split();
                touched_shrines.firefall_river_fire_shrine = true;
            }
        }
        6 => {if !touched_shrines.steam_house_fire_shrine {
                split();
                touched_shrines.steam_house_fire_shrine = true;
            }
        }
        _ => {}
    }
}
 */

const SHC_SHRINE: usize = 0;
const AP_SHRINE: usize = 1;
const TG_SHRINE: usize = 2;
const AT_SHRINE: usize = 3;
const FK_SHRINE: usize = 4;
const FFR_SHRINE: usize = 5;
const SH_SHRINE: usize = 6;

