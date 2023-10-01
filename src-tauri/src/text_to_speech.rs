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

use tts::{Features, Tts};

pub fn speak_string(text: String) {
    let (speaking_channel_tx, speaking_channel_rx) = channel();
    let speaking_channel_tx_clone = Arc::new(Mutex::new(speaking_channel_tx));

    let mut tts = Tts::default()?;
    let Features {
        utterance_callbacks,
        ..
    } = tts.supported_features();

    if utterance_callbacks {
        tts.on_utterance_begin(Some(Box::new(|utterance| {
            println!("Started speaking {:?}", utterance)
        })))?;
        tts.on_utterance_end(Some(Box::new(move |utterance| {
            println!("Finished speaking {:?}", utterance);
            speaking_channel_tx_clone.lock().unwrap().send(true).unwrap();
        })))?;
    }

    tts.speak(text, false)?;
    #[cfg(target_os = "macos")]
    {
        let run_loop: id = unsafe { NSRunLoop::currentRunLoop() };
        unsafe {
            let date: id = msg_send![class!(NSDate), distantFuture];
            let _: () = msg_send![run_loop, runMode:NSDefaultRunLoopMode beforeDate:date];
        }
    }
    speaking_channel_rx.recv().unwrap();
    println!("speaking done");
}