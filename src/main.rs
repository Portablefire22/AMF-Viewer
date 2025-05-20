mod amf;

use dioxus::desktop::tao::dpi::Size;
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::{tao, LogicalSize};
use dioxus::dioxus_core::SpawnIfAsync;
use dioxus::html::completions::CompleteWithBraces::p;
use dioxus::prelude::*;
use native_dialog::{DialogBuilder, MessageLevel};
use rfd::FileDialog;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

struct OpenedFile {
    is_open: bool,
    path: PathBuf,
}

impl OpenedFile {
    pub fn new() -> Self {
        OpenedFile {
            is_open: false,
            path: PathBuf::new(),
        }
    }
}

static CURRENT_FILE: GlobalSignal<OpenedFile> = Global::new(|| OpenedFile::new());

fn main() {
    let window = tao::window::WindowBuilder::new()
        .with_resizable(true)
        .with_min_inner_size(Size::Logical(LogicalSize {
            width: 1280.0,
            height: 720.0,
        }))
        .with_title("AMF Viewer");
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(window)
                .with_menu(None),
        )
        .launch(App);
}

fn show_error(title: &str, body: String) {
    DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_text(body)
        .alert()
        .spawn()
        .show()
        .unwrap();
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div {
            class: "ctp-latte dark:ctp-mocha bg-ctp-base min-w-full min-h-screen flex flex-row",
            LeftBar {}
            CentreBox {}
            RightBar {}
        }
    }
}

#[component]
fn LeftBar() -> Element {
    rsx! {
        div {
            class: "bg-ctp-crust grow-[3] m-3 outline outline-2 outline-ctp-pink p-2 rounded-md",
            button {
                class: "bg-ctp-surface0 outline outline-2 outline-ctp-pink text-ctp-text hover:outline-ctp-blue hover:bg-ctp-lavender hover:text-ctp-crust py-2 px-4 rounded",
                onclick: move |_| {
                    let mut handle = CURRENT_FILE.write();
                    handle.is_open = false;
                    handle.path = PathBuf::new();
                    let path = FileDialog::new().pick_file();
                    let path = match path {
                        Some(path) => path,
                        None => {
                            show_error("Error: Could not open file",
                                format!("Could not open file:\n{:?}", path));
                            return
                        }
                    };

                    let file = File::open(path.clone());
                    let file = match file {
                        Ok(file) => file,
                        Err(e) => {
                            show_error("Error: Could not open file",
                                format!("Could not open file:\n{:?}", e));
                            return
                        }
                    };
                    handle.is_open = true;
                    handle.path = path;
                },
                "Open File"
            }
        }
    }
}

#[component]
fn CentreBox() -> Element {
    let file = CURRENT_FILE.read();
    rsx! {
        div {
            class: "bg-ctp-crust m-3 grow-[7] max-w-[66%] outline outline-2 outline-ctp-pink p-2 rounded-md scroll-auto",
            h1 {
                class: "text-ctp-text",
                "Centre"
            }
            if file.is_open {
                FileOpened {}
            }
        }
    }
}

#[component]
fn FileOpened() -> Element {
    let file = CURRENT_FILE.read();
    let buffer = match fs::read(&file.path) {
        Ok(buffer) => buffer,
        Err(e) => {
            show_error(
                "Error: Could not read file",
                format!("Could not read file contents\n{:?}", e),
            );
            return rsx! {};
        }
    };

    rsx! {
        div {
            class: "max-w-[27rem]",

            span {
                class: "text-ctp-subtext0 hex",
                "00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F "
            }

            amf::amf_highlight::highlight_bytes {buffer, is_command: false}
        }
    }
}

#[component]
fn RightBar() -> Element {
    rsx! {
        div {
            class: "bg-ctp-crust min-w-1/5 grow-[3] m-3 outline outline-2 outline-ctp-pink p-2 rounded-md",
             h1 {
                class: "text-ctp-text",
                "Right"
            }
        }
    }
}

// /// Echo component that demonstrates fullstack server functions.
// #[component]
// fn Echo() -> Element {
//     let mut response = use_signal(|| String::new());
//
//     rsx! {
//         div {
//             id: "echo",
//             h4 { "ServerFn Echo" }
//             input {
//                 placeholder: "Type here to echo...",
//                 oninput:  move |event| async move {
//                     let data = echo_server(event.value()).await.unwrap();
//                     response.set(data);
//                 },
//             }
//
//             if !response().is_empty() {
//                 p {
//                     "Server echoed: "
//                     i { "{response}" }
//                 }
//             }
//         }
//     }
// }
//
// /// Echo the user input on the server.
// #[server(EchoServer)]
// async fn echo_server(input: String) -> Result<String, ServerFnError> {
//     Ok(input)
// }
