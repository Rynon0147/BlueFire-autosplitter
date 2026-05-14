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
//    string::ArrayCString,
    Address64,
    string::ArrayWString,
};
use alloc::{
    string::String,
    format,
};

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
    gruh_dead: bool,
    croh_dead: bool,
    sirion_dead: bool,
    beira_dead: bool,
    samael_dead: bool,
    queen_dead: bool,

    other_settings: Title,
    show_completion: bool,
}

const DEBUG: bool = false;

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
                    if DEBUG {set_variable_int("EventSize", size);}
                    watch_current_event_size.update_infallible(size);
                    if watch_current_event_size.pair.unwrap().changed() && size > 1 {

                        let offset = (size - 1) * 0x10;
                        if DEBUG {set_variable_int("Event offset" ,offset);}
                        if let Ok(event) = process.read_pointer_path::<u32>(
                            module.g_world(), 
                            Bit64, 
                            &offsets.events_array,
                        ) {
                            if DEBUG {set_variable_int("Events", event);}
                            let black_magic = event + offset;
                            if DEBUG {set_variable_int("black magic", black_magic);}
                            let magic_address = Address64::new(black_magic.into());
                            if let Ok(event_name_address) = process.read_pointer(magic_address, Bit64){
                                if DEBUG {set_variable_int("Event string address", event_name_address.value());}
                                if let Ok(event_string) = process.read::<ArrayWString<255>>(event_name_address){
                                    let plsmanijustwantastr: String = String::from_utf16_lossy(&event_string);
                                    let str_event: &str = plsmanijustwantastr.as_str();
                                    if DEBUG {set_variable("Event", str_event);}
                                    watch_current_event.update_infallible(plsmanijustwantastr);
                                }
                            }
                        }
                    }

                }
                
                // Cutscene (dont need yet~)
                /*
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
                */
                
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
                
                set_variable_int("FK state", split_states[4]);

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
                                        split_setting_check(AP_SOUL_FRAGMENTS_END, settings.ap_soul_fragments_end, &mut split_states, "AP frag end");
                                    }
                                    "BP_BeiraVesselBase_TempleGardens" => {
                                        split_setting_check(TG_SOUL_FRAGMENTS_END, settings.tg_soul_fragments_end, &mut split_states, "TG frag end");
                                    }
                                    "BP_BeiraVesselBase_LakeMolva" => {
                                        split_setting_check(FFR_SOUL_FRAGMENTS_END, settings.ffr_soul_fragments_end, &mut split_states, "FFR frag end");
                                    }
                                    "BP_BeiraVesselBase_GameIntro" =>{
                                        split_setting_check(FK_SOUL_FRAGMENTS_END, settings.fk_soul_fragments_end, &mut split_states, "FK frag end");
                                    }
                                    //bosses
                                    "NuosTempleEndCutscene" => {
                                        split_setting_check(GRUH, settings.gruh_dead, &mut split_states, "Gruh dead");
                                    }
                                    "UthasEndCutscene" => {
                                        split_setting_check(CROH, settings.croh_dead, &mut split_states, "Croh dead");
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
                                                    split_setting_check(SIRION, settings.sirion_dead, &mut split_states, "Sirion dead");
                                                }
                                                2u32 =>{
                                                    split_setting_check(BEIRA, settings.beira_dead, &mut split_states, "Beira dead");
                                                }
                                                9u32 => {
                                                    split_setting_check(SAMAEL, settings.samael_dead, &mut split_states, "Samael dead");
                                                }
                                                _ => {}                                     
                                            }
                                        }
                                    }
                                    "BossQueen" => {
                                        split_setting_check(QUEEN, settings.queen_dead, &mut split_states, "Queen dead");
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
        split();
        split_states[index] = 1;
        if DEBUG {set_variable_int(var_string, split_states[index]);print_message(var_string)}
    }

}


fn reset_all(split_states: &mut [i32;32]){
    split_states.fill(0);
 
    reset();
    start();
    pause_game_time();
    
    if DEBUG {
        set_variable_int("FK shrine", 0);
        set_variable_int("AT shrine", 0);
        set_variable_int("SHC shrine", 0);
        set_variable_int("AP shrine", 0);
        set_variable_int("TG shrine", 0);
        set_variable_int("FFR shrine", 0);
        set_variable_int("SH shrine", 0);

        set_variable_int("AP frag end", 0);
        set_variable_int("TG frag end", 0);
        set_variable_int("FFR frag end", 0);
        set_variable_int("FK frag end", 0);
    
        set_variable_int("Gruh dead", 0);
        set_variable_int("Croh dead", 0);
        set_variable_int("Sirion dead", 0);
        set_variable_int("Beira dead", 0);
        set_variable_int("Samael dead", 0);
        set_variable_int("Queen dead", 0);
    }
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

//bosses
const GRUH: usize = 11;
const CROH: usize = 12;
const SIRION: usize = 13;
const BEIRA: usize = 14;
const SAMAEL: usize = 15;
const QUEEN: usize = 16;
