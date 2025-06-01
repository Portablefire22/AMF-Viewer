mod amf;

use crate::amf::amf_highlight::AMFReader;
use std::collections::HashMap;

use crate::amf::object_info::ObjectInfo;
use crate::amf::object_properties::TypeProperties;
use crate::amf::object_type::ObjectType;
use dioxus::desktop::tao::dpi::Size;
use dioxus::desktop::{tao, LogicalSize};
use dioxus::dioxus_core::SpawnIfAsync;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use native_dialog::{DialogBuilder, MessageLevel};
use rfd::FileDialog;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

struct OpenedFile {
    is_open: bool,
    path: PathBuf,
    is_command: Signal<bool>,
}

impl OpenedFile {
    pub fn new() -> Self {
        OpenedFile {
            is_open: false,
            path: PathBuf::new(),
            is_command: Signal::new(false),
        }
    }
}

#[derive(Clone, Debug)]
struct ObjectContext {
    objects: Signal<HashMap<isize, ObjectInfo>>,
    selected_index: Signal<isize>,
    has_selected: Signal<bool>,
}

impl ObjectContext {
    pub fn new() -> Self {
        Self {
            objects: Signal::new(HashMap::new()),
            selected_index: Signal::new(0),
            has_selected: Signal::new(false),
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
        .with_title(format!("AMF Viewer v{}", env!("CARGO_PKG_VERSION")));
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
    let state = use_context_provider(|| ObjectContext::new());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        div {
            class: "ctp-latte dark:ctp-mocha bg-ctp-base min-w-full max-h-screen min-h-screen flex flex-row",
            LeftBar {}
            CentreBox {}
            RightBar {}
        }
    }
}

#[component]
fn LeftBar() -> Element {
    let mut command_signal = CURRENT_FILE.read().is_command;

    rsx! {
        div {
            class: "bg-ctp-crust grow-[3] m-3 outline outline-2 outline-ctp-pink p-2 rounded-md flex flex-row",
            div {
                div {
                    class: "h-fit m-2",
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
                            let mut obj_context = use_context::<ObjectContext>();
                            obj_context.objects.set(HashMap::new());
                            obj_context.has_selected.set(false);
                            obj_context.selected_index.set(0);

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
                div {
                    class: "h-fit m-2",
                    input {
                        r#type: "checkbox",
                        checked: command_signal,
                        oninput: move |_| {
                            let mut obj_context = use_context::<ObjectContext>();
                            obj_context.selected_index.set(0);
                            command_signal.set(!command_signal())
                        },
                    }
                    label {
                        class: "pl-2 text-ctp-text",
                        "Is command?"
                    }
                }
            }
        }
    }
}

#[component]
fn CentreBox() -> Element {
    let file = CURRENT_FILE.read();
    rsx! {
        div {
            class: "bg-ctp-crust m-3 grow-[7] max-w-[66%] outline outline-2 outline-ctp-pink p-2 rounded-md scroll-auto overflow-auto",
            h1 {
                class: "text-ctp-text",
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

    let mut reader = AMFReader::new(&buffer, *CURRENT_FILE.read().is_command.read());
    reader.highlight();

    let mut obj_context = use_context::<ObjectContext>();
    obj_context.objects.set(reader.objects.clone());
    obj_context.has_selected.set(true);

    rsx! {
        div {
            class: "max-w-[27rem]",

            span {
                class: "text-ctp-subtext0 hex",
                "00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F "
            }
            for byte in reader.out {
                span {
                    class: {
                        if byte.object_id == *obj_context.selected_index.read() {
                            format!("{} hex outline outline-2", byte.color)
                        } else {
                            format!("{} hex", byte.color)
                        }
                    },
                    id: "{byte.object_id}",
                    onclick:  move |_| {
                        tracing::debug!("Hovered: {:?}", byte.object_id);
                        obj_context.selected_index.set(byte.object_id);
                    },
                    "{byte.value:02X} "
                }
            }
        }
    }
}

#[component]
pub fn object_inspector(obj: Option<ObjectInfo>) -> Element {
    let obj = match obj {
        Some(obj) => obj,
        None => {
            return rsx! {
                "No object selected"
            }
        }
    };

    rsx! {
        div {
            class: "flex flex-col",
            TypeInspectorValue {name: "Object ID", value: obj.object_id}
            TypeInspectorValue {name: "Object Type", value: obj.object_type.clone()}
            TypeInspectorProperties {properties: obj.object_properties}
            type_inspector_contents {obj: obj.object_type}
        }
    }
}

#[component]
fn type_inspector_contents(obj: ObjectType, name: Option<String>) -> Element {
    let name = name.unwrap_or_else(|| String::from("Value"));
    match obj {
        ObjectType::Amf0Number(value) => rsx! {TypeInspectorValue {name, value}},
        ObjectType::Amf0Bool(value) => rsx! {TypeInspectorValue {name, value}},
        ObjectType::Amf0String(value) => rsx! {TypeInspectorValue {name, value}},
        ObjectType::Amf3Integer(value) => rsx! {TypeInspectorValue {name, value}},
        ObjectType::Amf3Double(value) => rsx! {TypeInspectorValue {name, value}},
        ObjectType::Amf3String(value) => rsx! {TypeInspectorValue {name, value}},
        ObjectType::Amf3Array(value) => {
            rsx! {TypeInspectorValue {name, value: format!("{:?}", value) }}
        }
        ObjectType::Amf3Object(value) => {
            rsx! {ObjectInspector {obj: value}}
        }
        ObjectType::Amf3Undefined | ObjectType::Amf0Undefined => rsx! {
            TypeInspectorValue {name, value: "Undefined"}
        },

        _ => rsx! {},
    }
}

#[component]
fn ObjectInspector(obj: HashMap<String, Option<isize>>) -> Element {
    let obj_context = use_context::<ObjectContext>();
    let handle = obj_context.objects.read();
    rsx! {
        h1 {
            class: "text-ctp-text font-bold",
            "Children"
        }
        for (key, id) in obj.iter() {
            type_inspector_contents{obj: {
                let x = handle.get(&id.unwrap()).unwrap().clone().object_type;
                tracing::debug!("Key: {} | Value: {:?}", key, x);
                x
            }, name: key.clone()}
        }
    }
}

#[component]
fn TypeInspectorProperties(properties: TypeProperties) -> Element {
    match properties {
        TypeProperties::Amf3StringProperties(prop) => {
            if prop.is_reference {
                rsx! {
                    TypeInspectorValue {name: "Is Reference?", value: prop.is_reference}
                    TypeInspectorValue {name: "Identifier", value: prop.identifier}
                }
            } else {
                rsx! {
                    TypeInspectorValue {name: "Is Reference?", value: prop.is_reference}
                    TypeInspectorValue {name: "String Length", value: prop.identifier}
                }
            }
        }
        TypeProperties::Amf3ObjectProperties(prop) => {
            rsx! {
                TypeInspectorValue {name: "Object Name", value: prop.object_type}
                TypeInspectorValue {name: "Is Reference?", value: prop.is_reference}
                TypeInspectorValue {name: "Property Count", value: prop.property_count}
                TypeInspectorValue {name: "Encoding", value: prop.encoding}
                TypeInspectorValue {name: "Externalisable", value: prop.externalisable}
                TypeInspectorValue {name: "Dynamic", value: prop.dynamic}
            }
        }
        _ => rsx! {},
    }
}

#[component]
fn TypeInspectorValue(name: String, value: String) -> Element {
    rsx! {
         span {
            class: "flex flex-row",
            p {
                class: "text-ctp-text font-medium",
                "{name}: "
            }
            p {
                class: "text-ctp-text pl-2",
                "{value}"
            }
        }
    }
}

#[component]
fn RightBar() -> Element {
    let cont = use_context::<ObjectContext>();
    let current_index = cont.selected_index.read();
    let obj = match *cont.has_selected.read() {
        true => {
            let obj = &cont.objects.read()[&current_index.clone()];
            Some(obj.clone())
        }
        false => None,
    };
    rsx! {
        div {
            class: "bg-ctp-crust min-w-1/5 grow-[3] max-w-[25%] m-3 outline outline-2 outline-ctp-pink p-2 rounded-md overflow-auto text-ctp-text",
            h1 {
                class: "text-ctp-text font-bold",
                "Object Inspector"
            }
            object_inspector {obj}
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
