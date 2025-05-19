use dioxus::desktop::tao::dpi::Size;
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::{tao, LogicalSize};
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

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

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
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
            h1 {
                class: "text-ctp-text",
                "Left"
            }
        }
    }
}

#[component]
fn CentreBox() -> Element {
    rsx! {
        div {
            class: "bg-ctp-crust grow-[7] m-3 outline outline-2 outline-ctp-pink p-2 rounded-md",
            h1 {
                class: "text-ctp-text",
                "Centre"
            }
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
