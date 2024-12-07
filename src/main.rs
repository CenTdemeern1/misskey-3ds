#![allow(dead_code)]
use std::{collections::HashSet, io::Write, net::TcpStream};

use ctru::{applets::{error::set_panic_hook, swkbd::{self, Button, ButtonConfig, Features, SoftwareKeyboard}}, prelude::*};
use http::{Message, Method};
use serde::Serialize;

mod http;

static_toml::static_toml! {
    const CONFIG = include_toml!("mk-config.toml");
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
enum Visibility {
    #[default]
    Public,
    Home,
    Followers,
    Specified
}

#[derive(Debug, Serialize)]
enum ReactionAcceptance {
    LikeOnly,
    LikeOnlyForRemote,
    NonSensitiveOnly,
    NonSensitiveOnlyForLocalLikeOnlyForRemote
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct Post {
    visibility: Visibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    visible_user_ids: Option<HashSet<String>>,
    cw: Option<String>,
    local_only: bool,
    reaction_acceptance: Option<ReactionAcceptance>,
    no_extract_mentions: bool,
    no_extract_hashtags: bool,
    no_extract_emojis: bool,
    reply_id: Option<String>,
    renote_id: Option<String>,
    channel_id: Option<String>,
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_ids: Option<HashSet<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_ids: Option<HashSet<String>>,
    poll: Option<bool>,
}

fn create_post(post: &Post) -> Message {
    let json = serde_json::to_string(post).unwrap();
    let mut request = Message::new_request(Method::POST, "/api/notes/create");
    request.set_header("Content-Type", "application/json");
    request.set_header("User-Agent", "misskey-3ds");
    request.set_header("Authorization", format!("Bearer {}", CONFIG.token));
    request.set_header("Host", CONFIG.host);
    request.set_content(Some(json.into_bytes().into_boxed_slice()));
    request
}

fn wait_to_exit(apt: &Apt, gfx: &Gfx, hid: &mut Hid) {
    println!("\x1b[29;16HPress Start to exit");

    while apt.main_loop() {
        gfx.wait_for_vblank();

        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
    }
}

fn main() {
    let apt = Apt::new().unwrap();
    let mut hid = Hid::new().unwrap();
    let gfx = Gfx::new().unwrap();
    let _console = Console::new(gfx.top_screen.borrow_mut());
    let _soc = Soc::new().unwrap();
    // if let Err(e) = soc.redirect_to_3dslink(true, true) {
    //     println!("Could not connect to 3dslink server: {}", e);
    //     wait_to_exit(&apt, &gfx, &mut hid);
    // }

    set_panic_hook(false);

    let mut keyboard = SoftwareKeyboard::new(swkbd::Kind::Normal, ButtonConfig::LeftRight);
    keyboard.configure_button(Button::Left, "Cancel", false);
    keyboard.configure_button(Button::Right, "Submit", true);
    keyboard.set_features(Features::MULTILINE | Features::PREDICTIVE_INPUT);

    let mut tcp = match TcpStream::connect((CONFIG.host, CONFIG.port as u16)) {
        Ok(tcp) => tcp,
        Err(err) => {
            eprintln!("Couldn't connect: {err}");
            wait_to_exit(&apt, &gfx, &mut hid);
            return;
        }
    };
    
    while let Ok((text, Button::Right)) = keyboard.launch(&apt, &gfx) {
        let post = create_post(&Post {
            visibility: Visibility::Public,
            text: Some(text),
            ..Default::default()
        });
        println!("{}", post);
        tcp.write_all(&post.serialize()).unwrap();
        tcp.flush().unwrap();
    }

    wait_to_exit(&apt, &gfx, &mut hid);
}
