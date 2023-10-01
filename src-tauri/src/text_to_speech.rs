use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

#[cfg(target_os = "macos")]
use cocoa_foundation::base::id;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSDefaultRunLoopMode;
#[cfg(target_os = "macos")]
use cocoa_foundation::foundation::NSRunLoop;
#[cfg(target_os = "macos")]
use objc::class;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
use once_cell::sync::OnceCell;

use tts::{Features, Tts};

pub static APP_TTS: OnceCell<Tts> = OnceCell::new();

pub fn speak_string(text: String) {

    if APP_TTS.get().is_none() {
        let init_tts = Tts::default().unwrap();
        APP_TTS.set(init_tts).unwrap_or_else(|_| panic!("Failed to set APP_TTS"));
    }
    let tts = APP_TTS.get().expect("APP_TTS not initialized");


    let (speaking_channel_tx, speaking_channel_rx) = channel();
    let speaking_channel_tx_clone = Arc::new(Mutex::new(speaking_channel_tx));

    let Features {
        utterance_callbacks,
        ..
    } = tts.supported_features();

    if utterance_callbacks {
        tts.on_utterance_begin(Some(Box::new(|utterance| {
            println!("Started speaking {:?}", utterance)
        }))).unwrap();
        tts.on_utterance_end(Some(Box::new(move |utterance| {
            println!("Finished speaking {:?}", utterance);
            speaking_channel_tx_clone.lock().unwrap().send(true).unwrap();
        }))).unwrap();
    }

    let mut local_tts = tts.clone();
    local_tts.speak(text, false).unwrap();
    #[cfg(target_os = "macos")]
    {
        let run_loop: id = unsafe { NSRunLoop::currentRunLoop() };
        unsafe {
            let date: id = msg_send![class!(NSDate), distantFuture];
            let _: () = msg_send![run_loop, runMode:NSDefaultRunLoopMode beforeDate:date];
        }
    }
    speaking_channel_rx.recv().unwrap();
}