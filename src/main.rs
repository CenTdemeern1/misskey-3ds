use std::{collections::HashSet, io::{Read, Write}, net::TcpStream};

use ctru::{applets::swkbd::{self, Button, ButtonConfig, Features, SoftwareKeyboard}, prelude::*};
use serde::Serialize;

const TOKEN: &'static str = "[redacted]";

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

fn create_post(post: &Post) -> String {
    let json = serde_json::to_string_pretty(post).unwrap();
    format!("POST /api/notes/create HTTP/1.1\r
Content-Type: application/json\r
User-Agent: misskey-3ds\r
Authorization: Bearer {TOKEN}\r
Host: eepy.moe\r
Content-Length: {}\r
\r
{json}", json.len())
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
    let mut sockets = Soc::new().unwrap();
    // sockets.redirect_to_3dslink(true, true).unwrap();

    let mut keyboard = SoftwareKeyboard::new(swkbd::Kind::Normal, ButtonConfig::LeftRight);
    keyboard.configure_button(Button::Left, "Cancel", false);
    keyboard.configure_button(Button::Right, "Submit", true);
    keyboard.set_features(Features::MULTILINE | Features::PREDICTIVE_INPUT);

    let mut tcp = match TcpStream::connect("192.168.178.89:80") {
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
        tcp.write_all(post.as_bytes()).unwrap();
        tcp.flush().unwrap();
        // tcp.shutdown(std::net::Shutdown::Write).unwrap();
        // let mut s = String::new();
        // tcp.read_to_string(&mut s).unwrap();
        // println!("{}", s);
    }

    wait_to_exit(&apt, &gfx, &mut hid);
}
