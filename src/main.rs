use leptos::prelude::*;
use leptos::html::Div;
use leptos_router::hooks::use_query_map;
use leptos_router::components::{Router, Routes, Route};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use web_sys::{FileSystemFileHandle, FileSystemWritableFileStream, File, Blob};
use js_sys::{ArrayBuffer, Uint8Array};

use gloo_storage::{Storage, LocalStorage};

mod models;
mod disasm;
mod database;

use crate::models::{DisassemblyInfo, DisassemblyLine, ThemeConfig};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = showOpenFilePicker, catch)]
    fn show_open_file_picker(options: &JsValue) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(js_name = showSaveFilePicker, catch)]
    fn show_save_file_picker(options: &JsValue) -> Result<js_sys::Promise, JsValue>;
}

#[derive(Clone)]
struct AppState {
    db: RwSignal<Option<DisassemblyInfo>>,
    db_handle: RwSignal<Option<FileSystemFileHandle>>,
    rom_data: RwSignal<Option<Vec<u8>>>,
    current_bank: RwSignal<u8>,
    active_theme: RwSignal<String>,
    themes: RwSignal<BTreeMap<String, ThemeConfig>>,
    
    // Resizing state
    col_widths: RwSignal<BTreeMap<String, i32>>,
    resizing: RwSignal<Option<String>>,
    start_x: RwSignal<i32>,
    start_width: RwSignal<i32>,
}

#[component]
fn App() -> impl IntoView {
    let db = RwSignal::new(None::<DisassemblyInfo>);
    let db_handle = RwSignal::new(None::<FileSystemFileHandle>);
    let rom_data = RwSignal::new(None::<Vec<u8>>);
    let current_bank = RwSignal::new(0u8);
    let active_theme = RwSignal::new(LocalStorage::get::<String>("activeTheme").unwrap_or_else(|_| "Dark".to_string()));
    
    // Effect to save theme preference
    Effect::new(move || {
        let _ = LocalStorage::set("activeTheme", active_theme.get());
    });
    
    let mut default_themes = BTreeMap::new();
    // ... default_themes setup ...
    default_themes.insert("Light".to_string(), ThemeConfig {
        name: "Light".to_string(),
        background: "#FFFFFF".to_string(),
        text: "#000000".to_string(),
        address: "#808080".to_string(),
        hex: "#808080".to_string(),
        instruction: "#000000".to_string(),
        opcode: "#000000".to_string(),
        comment: "#808080".to_string(),
        symbol: "#268bd2".to_string(),
    });
    default_themes.insert("Dark".to_string(), ThemeConfig {
        name: "Dark".to_string(),
        background: "#202020".to_string(),
        text: "#FFFFFF".to_string(),
        address: "#909090".to_string(),
        hex: "#909090".to_string(),
        instruction: "#FFFFFF".to_string(),
        opcode: "#FFFFFF".to_string(),
        comment: "#909090".to_string(),
        symbol: "#268bd2".to_string(),
    });
    let themes = RwSignal::new(default_themes);

    let mut initial_widths = BTreeMap::new();
    initial_widths.insert("addr".to_string(), 100);
    initial_widths.insert("hex".to_string(), 200);
    initial_widths.insert("op".to_string(), 60);
    initial_widths.insert("operand".to_string(), 150);
    let col_widths = RwSignal::new(initial_widths);
    let resizing = RwSignal::new(None::<String>);
    let start_x = RwSignal::new(0);
    let start_width = RwSignal::new(0);

    let state = AppState {
        db,
        db_handle,
        rom_data,
        current_bank,
        active_theme,
        themes,
        col_widths,
        resizing,
        start_x,
        start_width,
    };
    provide_context(state.clone());
    
    let on_mousemove = move |ev: web_sys::MouseEvent| {
        if let Some(col) = resizing.get() {
            let diff = ev.page_x() - start_x.get();
            let new_width = (start_width.get() + diff).max(20);
            col_widths.update(|w| {
                w.insert(col, new_width);
            });
        }
    };

    let on_mouseup = {
        let state = state.clone();
        move |_| {
            if let Some(_) = resizing.get_untracked() {
                if let Some(ref db) = state.db.get_untracked() {
                    let key = format!("{}.colWidths", db.name);
                    let _ = LocalStorage::set(&key, state.col_widths.get_untracked());
                }
                resizing.set(None);
            }
        }
    };

    // Effect to load saved column widths once DB is available
    Effect::new({
        let state = state.clone();
        move || {
            if let Some(db) = state.db.get() {
                let key = format!("{}.colWidths", db.name);
                if let Ok(saved) = LocalStorage::get::<BTreeMap<String, i32>>(&key) {
                    state.col_widths.set(saved);
                }
            }
        }
    });

    view! {
        <Router>
            <main on:mousemove=on_mousemove on:mouseup=on_mouseup class=move || if resizing.get().is_some() { "resizing" } else { "" }>
                <Routes fallback=|| view! { "Not Found" }>
                    <Route path=leptos_router::path!("/") view=MainContent />
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn MainContent() -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");
    // ... MainContent logic ...
    let query = use_query_map();

    // Effect to handle URL parameters
    let db_state = state.db;
    Effect::new(move || {
        let db_url = query.get().get("db").map(|s| s.to_string());
        if let Some(url) = db_url {
            if db_state.get_untracked().is_none() {
                leptos::task::spawn_local(async move {
                    if let Ok(resp) = gloo_net::http::Request::get(&url).send().await {
                        if let Ok(text) = resp.text().await {
                            if let Ok(parsed) = database::parse_db(&text) {
                                db_state.set(Some(parsed));
                            }
                        }
                    }
                });
            }
        }
    });

    let is_ready = move || state.db.get().is_some() && state.rom_data.get().is_some();

    view! {
        <div class="app-container">
            <ThemeStyle />
            {move || {
                if !is_ready() {
                    view! { <SetupScreen /> }.into_any()
                } else {
                    view! { <DisasmView /> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ThemeStyle() -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");
    let theme_memo = Memo::new(move |_| {
        let active = state.active_theme.get();
        state.themes.get().get(&active).cloned().unwrap_or_else(|| state.themes.get().values().next().unwrap().clone())
    });

    view! {
        <style>
            {move || {
                let t = theme_memo.get();
                let w = state.col_widths.get();
                format!(
                    "body {{ background-color: {}; color: {}; \
                            --col-addr: {}px; --col-hex: {}px; --col-op: {}px; --col-operand: {}px; }}\n\
                     .header {{ background-color: {}; color: {}; border-bottom-color: {}; }}\n\
                     select {{ background-color: {}; color: {}; border-color: {}; }}\n\
                     .grid-header {{ background-color: {}; color: {}; border-color: {}; }}\n\
                     .grid-header .grid-cell {{ border-color: {}; }}\n\
                     .address {{ color: {}; }}\n\
                     .hex {{ color: {}; }}\n\
                     .instruction {{ color: {}; }}\n\
                     .opcode {{ color: {}; }}\n\
                     .operand {{ color: {}; }}\n\
                     .comment {{ color: {}; }}\n\
                     .symbol {{ color: {}; }}\n\
                     a {{ color: inherit; text-decoration: none; }}\n\
                     a.symbol {{ color: {}; }}\n\
                     a:hover {{ text-decoration: underline; }}\n",
                    t.background, t.instruction,
                    w.get("addr").unwrap_or(&100),
                    w.get("hex").unwrap_or(&200),
                    w.get("op").unwrap_or(&60),
                    w.get("operand").unwrap_or(&150),
                    t.background, t.text, t.address,
                    t.background, t.text, t.address,
                    t.background, t.text, t.address,
                    t.address,
                    t.address, t.hex, t.instruction, t.opcode,
                    t.instruction, t.comment, t.symbol, t.symbol
                )
            }}
        </style>
    }
}

#[component]
fn SetupScreen() -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");

    let open_db = {
        let state = state.clone();
        move |_| {
            let state = state.clone();
            leptos::task::spawn_local(async move {
                let options = js_sys::Object::new();
                let types = js_sys::Array::new();
                let type_info = js_sys::Object::new();
                let accept = js_sys::Object::new();
                let extensions = js_sys::Array::new();
                extensions.push(&JsValue::from_str(".json"));
                extensions.push(&JsValue::from_str(".json5"));
                js_sys::Reflect::set(&accept, &JsValue::from_str("application/json"), &extensions).unwrap();
                js_sys::Reflect::set(&type_info, &JsValue::from_str("description"), &JsValue::from_str("JSON Database")).unwrap();
                js_sys::Reflect::set(&type_info, &JsValue::from_str("accept"), &accept).unwrap();
                types.push(&type_info);
                js_sys::Reflect::set(&options, &JsValue::from_str("types"), &types).unwrap();

                if let Ok(promise) = show_open_file_picker(&options) {
                    if let Ok(handles_val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                        let handles: js_sys::Array = handles_val.unchecked_into();
                        if handles.length() > 0 {
                            let handle: FileSystemFileHandle = handles.get(0).unchecked_into();
                            state.db_handle.set(Some(handle.clone()));
                            let file_promise = handle.get_file();
                            if let Ok(file_val) = wasm_bindgen_futures::JsFuture::from(file_promise).await {
                                let file: File = file_val.unchecked_into();
                                let text_promise = file.text();
                                if let Ok(text_val) = wasm_bindgen_futures::JsFuture::from(text_promise).await {
                                    let text: String = text_val.as_string().unwrap();
                                    if let Ok(parsed) = database::parse_db(&text) {
                                        state.db.set(Some(parsed));
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    };

    let open_rom = {
        let state = state.clone();
        move |_| {
            let state = state.clone();
            leptos::task::spawn_local(async move {
                let options = js_sys::Object::new();
                let types = js_sys::Array::new();
                let type_info = js_sys::Object::new();
                let accept = js_sys::Object::new();
                let extensions = js_sys::Array::new();
                extensions.push(&JsValue::from_str(".nes"));
                js_sys::Reflect::set(&accept, &JsValue::from_str("application/octet-stream"), &extensions).unwrap();
                js_sys::Reflect::set(&type_info, &JsValue::from_str("description"), &JsValue::from_str("NES ROM")).unwrap();
                js_sys::Reflect::set(&type_info, &JsValue::from_str("accept"), &accept).unwrap();
                types.push(&type_info);
                js_sys::Reflect::set(&options, &JsValue::from_str("types"), &types).unwrap();

                if let Ok(promise) = show_open_file_picker(&options) {
                    if let Ok(handles_val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                        let handles: js_sys::Array = handles_val.unchecked_into();
                        if handles.length() > 0 {
                            let handle: FileSystemFileHandle = handles.get(0).unchecked_into();
                            let file_promise = handle.get_file();
                            if let Ok(file_val) = wasm_bindgen_futures::JsFuture::from(file_promise).await {
                                let file: File = file_val.unchecked_into();
                                let buffer_promise = file.array_buffer();
                                if let Ok(buffer_val) = wasm_bindgen_futures::JsFuture::from(buffer_promise).await {
                                    let buffer: ArrayBuffer = buffer_val.unchecked_into();
                                    let array = Uint8Array::new(&buffer);
                                    let bytes = array.to_vec();
                                    state.rom_data.set(Some(bytes));
                                }
                            }
                        }
                    }
                }
            });
        }
    };

    view! {
        <div class="setup-screen">
            <h1>"Docassembler Setup"</h1>
            {move || if state.db.get().is_none() {
                let open_db = open_db.clone();
                view! {
                    <div class="setup-step">
                        <p>"Please open your disassembly database (.json file)."</p>
                        <button on:click=open_db>"Open Database"</button>
                    </div>
                }.into_any()
            } else {
                let title = state.db.get().map(|d| d.title).unwrap_or_default();
                let open_rom = open_rom.clone();
                view! {
                    <div class="setup-step">
                        <p>"Database loaded: " <strong>{title}</strong></p>
                        <p>"Please provide the NES ROM file for this project."</p>
                        <button on:click=open_rom>"Open ROM"</button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn DisasmView() -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");

    let banks = move || {
        let mut b = BTreeMap::new();
        if let Some(ref d) = state.db.get() {
            let rom_data = state.rom_data.get().unwrap();
            let prg_banks_16k = rom_data[4];
            let num_banks = match d.mapper_window_size {
                8 => prg_banks_16k * 2,
                16 => prg_banks_16k,
                _ => prg_banks_16k,
            };
            for i in 0..num_banks {
                let title = d.bank.get(&i).and_then(|b| b.title.clone()).unwrap_or_default();
                b.insert(i, title);
            }
            b.insert(255, "Global Symbols".to_string());
        }
        b
    };

    let state_c = state.clone();
    let on_bank_change = move |ev| {
        let val = event_target_value(&ev);
        if let Ok(id) = val.parse::<u8>() {
            state_c.current_bank.set(id);
        }
    };

    let state_c2 = state.clone();
    let state_c3 = state.clone();
    view! {
        <div class="main-view">
            <header class="header">
                <h1>"Docassembler: " {move || state.db.get().map(|d| d.title).unwrap_or_default()}</h1>
                <div style="display: flex; gap: 20px; align-items: center;">
                    <div>
                        "Bank: "
                        <select on:change=on_bank_change prop:value=move || state.current_bank.get().to_string()>
                            {move || banks().into_iter().map(|(id, title)| {
                                view! { <option value=id.to_string()> {format!("${:02X}: {}", id, title)} </option> }
                            }).collect_view()}
                        </select>
                    </div>
                    <div>
                        "Theme: "
                        <select on:change={let state = state_c3.clone(); move |ev| state.active_theme.set(event_target_value(&ev))}>
                            {
                                let state_c3 = state_c3.clone();
                                move || {
                                    let state_outer = state_c3.clone();
                                    state_c3.themes.get().keys().cloned().collect::<Vec<_>>().into_iter().map(move |name| {
                                        let name_c = name.clone();
                                        let state_inner = state_outer.clone();
                                        view! { <option value=name.clone() selected=move || state_inner.active_theme.get() == name_c> {name.clone()} </option> }
                                    }).collect_view()
                                }
                            }
                        </select>
                    </div>
                    <button on:click=move |_| save_db_logic(state_c2.clone())>"Save"</button>
                </div>
                <div class="grid-header">
                    <div class="grid-cell" style="width: var(--col-addr)">"Addr"</div>
                    <div class="resizer" on:mousedown={let state = state.clone(); move |ev| start_resizing(state.clone(), "addr", ev)}></div>
                    <div class="grid-cell" style="width: var(--col-hex)">"Bytes"</div>
                    <div class="resizer" on:mousedown={let state = state.clone(); move |ev| start_resizing(state.clone(), "hex", ev)}></div>
                    <div class="grid-cell" style="width: var(--col-op)">"Op"</div>
                    <div class="resizer" on:mousedown={let state = state.clone(); move |ev| start_resizing(state.clone(), "op", ev)}></div>
                    <div class="grid-cell" style="width: var(--col-operand)">"Operand"</div>
                    <div class="resizer" on:mousedown={let state = state.clone(); move |ev| start_resizing(state.clone(), "operand", ev)}></div>
                    <div class="grid-cell">"Comment"</div>
                </div>
            </header>
            <VirtualizedDisasm />
        </div>
    }
}

fn start_resizing(state: AppState, col: &str, ev: web_sys::MouseEvent) {
    ev.prevent_default();
    state.resizing.set(Some(col.to_string()));
    state.start_x.set(ev.page_x());
    state.start_width.set(*state.col_widths.get_untracked().get(col).unwrap_or(&100));
}

fn save_db_logic(state: AppState) {
    leptos::task::spawn_local(async move {
        if let Some(db) = state.db.get_untracked() {
            if let Ok(json) = database::serialize_db(&db) {
                if let Some(handle) = state.db_handle.get_untracked() {
                    let promise = handle.create_writable();
                    if let Ok(writable_val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                        let writable: FileSystemWritableFileStream = writable_val.unchecked_into();
                        let blob = Blob::new_with_str_sequence(&js_sys::Array::of1(&json.into())).unwrap();
                        let _ = wasm_bindgen_futures::JsFuture::from(writable.write_with_blob(&blob).expect("write_with_blob failed")).await;
                        let _ = wasm_bindgen_futures::JsFuture::from(writable.close()).await;
                    }
                } else {
                    // Save As
                    let options = js_sys::Object::new();
                    js_sys::Reflect::set(&options, &JsValue::from_str("suggestedName"), &JsValue::from_str(&format!("{}.json", db.name))).unwrap();
                    if let Ok(promise) = show_save_file_picker(&options) {
                        if let Ok(handle_val) = wasm_bindgen_futures::JsFuture::from(promise).await {
                             let handle: FileSystemFileHandle = handle_val.unchecked_into();
                             state.db_handle.set(Some(handle.clone()));
                             // Recursive call or just repeat logic here
                             let writable_promise = handle.create_writable();
                             if let Ok(writable_val) = wasm_bindgen_futures::JsFuture::from(writable_promise).await {
                                let writable: FileSystemWritableFileStream = writable_val.unchecked_into();
                                let blob = Blob::new_with_str_sequence(&js_sys::Array::of1(&json.into())).unwrap();
                                let _ = wasm_bindgen_futures::JsFuture::from(writable.write_with_blob(&blob).expect("write_with_blob failed")).await;
                                let _ = wasm_bindgen_futures::JsFuture::from(writable.close()).await;
                             }
                        }
                    }
                }
            }
        }
    });
}

#[component]
fn VirtualizedDisasm() -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");
    let container_ref = NodeRef::<Div>::new();
    let (scroll_top, set_scroll_top) = RwSignal::new(0.0).split();
    let (viewport_height, set_viewport_height) = RwSignal::new(1000.0).split();

    // Effect to initialize viewport height and handle resize
    Effect::new({
        let state = state.clone();
        move || {
            if let Some(div) = container_ref.get() {
                set_viewport_height.set(div.client_height() as f64);
                
                let handle_resize = {
                    let div = div.clone();
                    move |_: web_sys::Event| {
                        set_viewport_height.set(div.client_height() as f64);
                    }
                };
                
                let window = web_sys::window().unwrap();
                let closure = Closure::wrap(Box::new(handle_resize) as Box<dyn FnMut(web_sys::Event)>);
                window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref()).unwrap();
                closure.forget();
            }
        }
    });

    // Effect to reset scroll on bank change
    Effect::new({
        move || {
            let _ = state.current_bank.get();
            if let Some(div) = container_ref.get() {
                div.set_scroll_top(0);
                set_scroll_top.set(0.0);
            }
        }
    });

    let disassembly = Memo::new(move |_| {
        let bank_id = state.current_bank.get();
        let db = state.db.get();
        let rom_data = state.rom_data.get();
        
        if let (Some(db), Some(rom_data)) = (db, rom_data) {
            let global_targets = disasm::discover_all_targets(&db, &rom_data);
            if bank_id == 255 {
                let mut lines = Vec::new();
                for (addr, anno) in &db.global {
                    lines.push(DisassemblyLine {
                        address: *addr,
                        address_label: format!("${:04X}", addr),
                        bank: -1,
                        bytes: String::new(),
                        opcode: String::new(),
                        operand_prefix: String::new(),
                        operand_main: String::new(),
                        operand_suffix: String::new(),
                        operand_is_symbol: false,
                        symbol: anno.symbol.clone(),
                        comment: anno.comment.clone(),
                        block_comment: anno.block_comment.clone(),
                        target_bank: None,
                        target_address: None,
                    });
                }
                lines
            } else {
                let mapper_size = db.mapper_window_size as usize * 1024;
                let rom_offset = 16 + (bank_id as usize * mapper_size);
                let rom_end = (rom_offset + mapper_size).min(rom_data.len());
                let bank_data = if rom_offset < rom_data.len() { &rom_data[rom_offset..rom_end] } else { &[] };
                disasm::disassemble_bank(&db, bank_id, bank_data, &global_targets)
            }
        } else {
            Vec::new()
        }
    });

    const LINE_HEIGHT: f64 = 20.0;

    let offsets = Memo::new(move |_| {
        let lines = disassembly.get();
        let mut current = 0.0;
        let mut off = Vec::with_capacity(lines.len());
        for line in &lines {
            off.push(current);
            let mut height = LINE_HEIGHT;
            if line.bank != -1 {
                if let Some(ref bc) = line.block_comment {
                    let count = bc.lines().count() as f64;
                    height += count * LINE_HEIGHT;
                }
                if line.symbol.is_some() {
                    height += LINE_HEIGHT;
                }
            }
            current += height;
        }
        (off, current)
    });

    let visible_range = move || {
        let (off, total) = offsets.get();
        if off.is_empty() { return (0, 0, 0.0); }

        let start_y = scroll_top.get();
        let end_y = start_y + viewport_height.get();
        
        let start_idx = match off.binary_search_by(|v| v.partial_cmp(&start_y).unwrap()) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        
        let end_idx = match off.binary_search_by(|v| v.partial_cmp(&end_y).unwrap()) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
        
        let buffer = 20;
        let start = start_idx.saturating_sub(buffer);
        let end = (end_idx + buffer).min(disassembly.get().len());
        (start, end, total)
    };

    let on_scroll = move |ev: web_sys::Event| {
        let div = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
        set_scroll_top.set(div.scroll_top() as f64);
        set_viewport_height.set(div.client_height() as f64);
    };

    view! {
        <div 
            class="disassembly-container" 
            node_ref=container_ref 
            on:scroll=on_scroll
            style="position: relative; overflow-y: auto; height: 100%;"
        >
            <div style=move || format!("height: {}px; position: relative;", offsets.get().1)>
                {move || {
                    let (start, end, _) = visible_range();
                    let lines = disassembly.get();
                    let (off, _) = offsets.get();
                    if start >= end || start >= lines.len() {
                        return view! {}.into_any();
                    }
                    lines[start..end].iter().enumerate().map(|(i, line)| {
                        let actual_idx = start + i;
                        let top = off[actual_idx];
                        view! { <DisasmRow line=line.clone() top=top /> }
                    }).collect_view().into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn DisasmRow(line: DisassemblyLine, top: f64) -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");

    let on_symbol_blur = {
        let state = state.clone();
        let line = line.clone();
        move |ev: web_sys::FocusEvent| {
            let val = event_target_inner_text(&ev);
            update_annotation(state.clone(), line.address, line.bank, "symbol", val);
        }
    };

    let on_comment_blur = {
        let state = state.clone();
        let line = line.clone();
        move |ev: web_sys::FocusEvent| {
            let val = event_target_inner_text(&ev);
            update_annotation(state.clone(), line.address, line.bank, "comment", val);
        }
    };

    let on_block_blur = {
        let state = state.clone();
        let line = line.clone();
        move |ev: web_sys::FocusEvent| {
            let val = event_target_inner_text(&ev);
            update_annotation(state.clone(), line.address, line.bank, "block_comment", val);
        }
    };

    let state_nav = state.clone();
    view! {
        <div class="grid-row" style=format!("position: absolute; top: {}px; width: 100%; display: grid; grid-template-columns: var(--col-addr) var(--col-hex) var(--col-op) var(--col-operand) 1fr;", top)>
            {if let Some(ref bc) = line.block_comment {
                view! {
                    <div class="grid-cell full-width" style="grid-column: 1 / -1;">
                        <div class="comment editable-container" contenteditable="true" on:blur=on_block_blur>
                            {bc.lines().map(|l| format!("; {}", l)).collect::<Vec<_>>().join("\n")}
                        </div>
                    </div>
                }.into_any()
            } else { view! {}.into_any() }}

            {if line.bank != -1 {
                let state_nav = state_nav.clone();
                view! {
                    {if let Some(ref sym) = line.symbol {
                        view! {
                            <div class="grid-cell full-width" style="grid-column: 1 / -1;">
                                <div class="symbol editable-container" contenteditable="true" on:blur=on_symbol_blur>
                                    {format!("{}:", sym)}
                                </div>
                            </div>
                        }.into_any()
                    } else { view! {}.into_any() }}
                    <div class="grid-cell address">{line.address_label}</div>
                    <div class="grid-cell hex">{line.bytes}</div>
                    <div class="grid-cell opcode">{line.opcode}</div>
                    <div class="grid-cell operand">
                        <span>{line.operand_prefix}</span>
                        {if let Some(target_addr) = line.target_address {
                            let target_bank = line.target_bank;
                            let state_nav = state_nav.clone();
                            view! {
                                <a href="#" on:click=move |e| { e.prevent_default(); navigate(state_nav.clone(), target_bank, target_addr); }>
                                    {line.operand_main.clone()}
                                </a>
                            }.into_any()
                        } else {
                            view! { <span>{line.operand_main.clone()}</span> }.into_any()
                        }}
                        <span>{line.operand_suffix}</span>
                    </div>
                    <div class="grid-cell comment-cell">
                        <div class="comment editable-container" contenteditable="true" on:blur=on_comment_blur>
                            {line.comment.map(|c| format!("; {}", c)).unwrap_or_default()}
                        </div>
                    </div>
                }.into_any()
            } else {
                // Global Equate
                view! {
                    <div class="grid-cell address" style="grid-column: 1 / span 4; display: flex; align-items: baseline;">
                        <div class="symbol editable-container" contenteditable="true" on:blur=on_symbol_blur>
                            {line.symbol.unwrap_or_else(|| "???".to_string())}
                        </div>
                        <span style="margin-left: 8px;">" = " {line.address_label}</span>
                    </div>
                    <div class="grid-cell comment-cell">
                        <div class="comment editable-container" contenteditable="true" on:blur=on_comment_blur>
                            {line.comment.map(|c| format!("; {}", c)).unwrap_or_default()}
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

fn update_annotation(state: AppState, address: u16, bank_id: i16, field: &str, value: String) {
    let mut db = state.db.get_untracked().unwrap();
    let bank_id_opt = if bank_id == -1 { None } else { Some(bank_id as u8) };

    let processed = strip_decorations(field, &value);
    
    if let Some(id) = bank_id_opt {
        let bank = db.bank.entry(id).or_insert_with(|| crate::models::BankInfo {
            title: None,
            is_fixed: false,
            mapped_at: Some(0x8000),
            region: Vec::new(),
            address: std::collections::BTreeMap::new(),
        });
        let section = bank.address.entry(address).or_default();
        match field {
            "symbol" => section.symbol = if processed.is_empty() { None } else { Some(processed) },
            "comment" => section.comment = if processed.is_empty() { None } else { Some(processed) },
            "block_comment" => section.block_comment = if processed.is_empty() { None } else { Some(processed) },
            _ => {}
        }
        if section.is_empty() {
            bank.address.remove(&address);
        }
    } else {
        let section = db.global.entry(address).or_default();
        match field {
            "symbol" => section.symbol = if processed.is_empty() { None } else { Some(processed) },
            "comment" => section.comment = if processed.is_empty() { None } else { Some(processed) },
            "block_comment" => section.block_comment = if processed.is_empty() { None } else { Some(processed) },
            _ => {}
        }
        if section.is_empty() {
            db.global.remove(&address);
        }
    }

    state.db.set(Some(db));
}

fn strip_decorations(field: &str, text: &str) -> String {
    let text = text.trim();
    if field == "symbol" {
        if text.ends_with(':') { return text[..text.len()-1].to_string(); }
        return text.to_string();
    }
    if field == "comment" || field == "block_comment" {
        return text.lines().map(|line| {
            let mut l = line.trim_start();
            if l.starts_with(';') {
                l = &l[1..];
                if l.starts_with(' ') { l = &l[1..]; }
            }
            l
        }).collect::<Vec<_>>().join("\n").trim_end().to_string();
    }
    text.to_string()
}

fn navigate(state: AppState, target_bank: Option<u8>, _target_address: u16) {
    if let Some(bank) = target_bank {
        state.current_bank.set(bank);
    }
}

fn event_target_inner_text(ev: &web_sys::FocusEvent) -> String {
    let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
    target.inner_text()
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
