use leptos::prelude::*;
use leptos::html::Div;
use leptos_router::hooks::use_query_map;
use leptos_router::components::{Router, Routes, Route};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;
use web_sys::{FileSystemFileHandle, FileSystemWritableFileStream, File, Blob};
use js_sys::{ArrayBuffer, Uint8Array};
use regex::Regex;

use gloo_storage::{Storage, LocalStorage};

use crate::models::{DisassemblyInfo, DisassemblyLine, ThemeConfig};
use crate::{database, disasm};

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
    
    // Navigation state
    nav_target: RwSignal<Option<u16>>,
    is_navigating: RwSignal<bool>,

    // Block comment editing state
    editing_block_comment: RwSignal<Option<(u16, i16)>>,
    editing_operand: RwSignal<Option<(u16, i16)>>,
    main_container_ref: NodeRef<Div>,

    // Search state
    search_query: RwSignal<String>,
    search_current_idx: RwSignal<usize>,
    disassembly: Memo<Vec<DisassemblyLine>>,
    search_results: Memo<Vec<u16>>,
}

#[component]
pub fn App() -> impl IntoView {
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
        highlight: "#ffd70066".to_string(),
        current_highlight: "#ffd700cc".to_string(),
        match_cell: "#ffd70022".to_string(),
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
        highlight: "#ffd70066".to_string(),
        current_highlight: "#ffd700cc".to_string(),
        match_cell: "#ffd70022".to_string(),
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
    let nav_target = RwSignal::new(None::<u16>);
    let is_navigating = RwSignal::new(false);
    let editing_block_comment = RwSignal::new(None::<(u16, i16)>);
    let editing_operand = RwSignal::new(None::<(u16, i16)>);
    let main_container_ref = NodeRef::<Div>::new();

    let search_query = RwSignal::new(String::new());
    let search_current_idx = RwSignal::new(0usize);

    let disassembly = Memo::new(move |_| {
        let bank_id = current_bank.get();
        let db = db.get();
        let rom_data = rom_data.get();
        
        if let (Some(db), Some(rom_data)) = (db, rom_data) {
            let bank_targets = disasm::discover_all_targets(&db, &rom_data);
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
                disasm::disassemble_bank(&db, bank_id, bank_data, &bank_targets)
            }
        } else {
            Vec::new()
        }
    });

    let search_results = Memo::new(move |_| {
        let query = search_query.get().to_lowercase();
        if query.is_empty() { return Vec::new(); }
        
        disassembly.get().iter()
            .filter(|line| {
                line.symbol.as_ref().map_or(false, |s| s.to_lowercase().contains(&query)) ||
                line.operand_main.to_lowercase().contains(&query) ||
                line.comment.as_ref().map_or(false, |c| c.to_lowercase().contains(&query)) ||
                line.block_comment.as_ref().map_or(false, |bc| bc.to_lowercase().contains(&query))
            })
            .map(|line| line.address)
            .collect::<Vec<_>>()
    });

    // Reset search index when results change
    Effect::new(move || {
        let _ = search_results.get();
        search_current_idx.set(0);
    });

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
        nav_target,
        is_navigating,
        editing_block_comment,
        editing_operand,
        main_container_ref,
        search_query,
        search_current_idx,
        disassembly,
        search_results,
    };
    provide_context(state.clone());

    // Hash handling logic
    let handle_hash = {
        let state = state.clone();
        move || {
            let window = web_sys::window().unwrap();
            let hash = window.location().hash().unwrap_or_default();
            if hash.is_empty() { return; }

            // Patterns: #bank-XX, #bank-XX-addr-XXXX
            let bank_re = Regex::new(r"bank-([0-9A-Fa-f]{2})").unwrap();
            let addr_re = Regex::new(r"addr-([0-9A-Fa-f]{4})").unwrap();

            let mut target_bank = None;
            let mut target_addr = None;

            if let Some(caps) = bank_re.captures(&hash) {
                let hex = caps.get(1).unwrap().as_str();
                let id = if hex == "FF" { 255 } else { u8::from_str_radix(hex, 16).unwrap_or(0) };
                target_bank = Some(id);
            }

            if let Some(caps) = addr_re.captures(&hash) {
                let hex = caps.get(1).unwrap().as_str();
                target_addr = u16::from_str_radix(hex, 16).ok();
            }

            if let Some(bank) = target_bank {
                if state.current_bank.get_untracked() != bank {
                    state.is_navigating.set(true);
                    state.current_bank.set(bank);
                }
            }

            if let Some(addr) = target_addr {
                state.is_navigating.set(true);
                state.nav_target.set(Some(addr));
            }
        }
    };

    // Effect to set up hashchange listener
    Effect::new({
        let handle_hash = handle_hash.clone();
        move || {
            let window = web_sys::window().unwrap();
            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                handle_hash();
            }) as Box<dyn FnMut(web_sys::Event)>);
            window.add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref()).unwrap();
            closure.forget();
            
            // Initial hash check
            handle_hash();
        }
    });
    
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

    let base = option_env!("TRUNK_PUBLIC_URL").unwrap_or("/").trim_end_matches('/');

    view! {
        <Router base=base.to_string()>
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

    let is_ready = Memo::new(move |_| state.db.get().is_some() && state.rom_data.get().is_some());

    view! {
        <div class="app-container">
            <ThemeStyle />
            <Show
                when=move || is_ready.get()
                fallback=|| view! { <SetupScreen /> }
            >
                <DisasmView />
            </Show>
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
                    "body {{ background-color: {}; color: {}; 
                            --col-addr: {}px; --col-hex: {}px; --col-op: {}px; --col-operand: {}px; }}

                     .header {{ background-color: {}; color: {}; border-bottom-color: {}; }}

                     select, input {{ background-color: {}; color: {}; border-color: {}; border-style: solid; border-width: 1px; padding: 2px 5px; }}

                     .grid-header {{ background-color: {}; color: {}; border-color: {}; }}

                     .grid-header .grid-cell {{ border-color: {}; }}

                     .address {{ color: {}; }}

                     .hex {{ color: {}; }}

                     .instruction {{ color: {}; }}

                     .opcode {{ color: {}; }}

                     .operand {{ color: {}; }}

                     .comment {{ color: {}; }}

                     .symbol {{ color: {}; }}

                     a {{ color: inherit; text-decoration: none; }}

                     a.symbol {{ color: {}; }}

                     a:hover {{ text-decoration: underline; }}

                     .search-highlight {{ background-color: {}; border-radius: 2px; }}
                     .search-highlight-active {{ background-color: {}; border-radius: 2px; }}
                     .search-match-cell {{ background-color: {}; }}
",
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
                    t.instruction, t.comment, t.symbol, t.symbol,
                    t.highlight, t.current_highlight, t.match_cell
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

    let load_remote_db = {
        let state = state.clone();
        move |url: String| {
            let state = state.clone();
            leptos::task::spawn_local(async move {
                if let Ok(resp) = gloo_net::http::Request::get(&url).send().await {
                    if let Ok(text) = resp.text().await {
                        if let Ok(parsed) = database::parse_db(&text) {
                            state.db.set(Some(parsed));
                        }
                    }
                }
            });
        }
    };

    let remote_dbs = move || {
        let env_val = option_env!("DOCASSEMBLER_DB").unwrap_or("");
        if env_val.is_empty() { return Vec::new(); }
        env_val.split(';')
            .filter_map(|s| {
                let parts: Vec<&str> = s.split('=').collect();
                if parts.len() == 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="setup-screen">
            <h1>"Docassembler Setup"</h1>
            {move || if state.db.get().is_none() {
                let open_db = open_db.clone();
                let load_remote_db = load_remote_db.clone();
                view! {
                    <div class="setup-step" style="display: flex; flex-direction: column; gap: 10px;">
                        <p>"Please open a disassembly database."</p>
                        {remote_dbs().into_iter().map(|(name, url)| {
                            let load_remote_db = load_remote_db.clone();
                            view! {
                                <button type="button" on:click=move |e| { e.prevent_default(); load_remote_db(url.clone()); }>
                                    {format!("{}", name)}
                                </button>
                            }
                        }).collect_view()}
                        <button type="button" on:click=move |e| { e.prevent_default(); open_db(e); }>"Open Local Database"</button>
                    </div>
                }.into_any()
            } else {
                let title = state.db.get().map(|d| d.title).unwrap_or_default();
                let open_rom = open_rom.clone();
                view! {
                    <div class="setup-step">
                        <p>"Database loaded: " <strong>{title}</strong></p>
                        <p>"Please provide the NES ROM file for this project."</p>
                        <button type="button" on:click=move |e| { e.prevent_default(); open_rom(e); }>"Open ROM"</button>
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
            let window = web_sys::window().unwrap();
            let bank_hex = format!("{:02X}", id);
            let hash = format!("#bank-{}", bank_hex);
            window.location().set_hash(&hash).unwrap();
        }
    };

    let state_c2 = state.clone();
    let state_c3 = state.clone();
    let state_search = state.clone();
    
    let on_search_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            ev.prevent_default();
            let results = state_search.search_results.get();
            if results.is_empty() { return; }
            
            let mut idx = state_search.search_current_idx.get();
            if ev.ctrl_key() {
                idx = if idx == 0 { results.len() - 1 } else { idx - 1 };
            } else {
                idx = (idx + 1) % results.len();
            }
            state_search.search_current_idx.set(idx);
            state_search.nav_target.set(Some(results[idx]));
        }
    };

    let help_url = option_env!("DOCASSEMBLER_HELP").unwrap_or("https://github.com/cfrantz/z2doc/blob/main/README.md");

    view! {
        <div class="main-view">
            <header class="header">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <h1>"Docassembler: " {move || state.db.get().map(|d| d.title).unwrap_or_default()}</h1>
                    <a href=help_url target="_blank">
                        <button type="button">"Help"</button>
                    </a>
                </div>
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
                        <input 
                            type="text" 
                            placeholder="Search..." 
                            style="width: 250px;"
                            prop:value=move || state_search.search_query.get()
                            on:input=move |ev| state_search.search_query.set(event_target_value(&ev))
                            on:keydown=on_search_keydown
                        />
                        <span style="margin-left: 5px; font-size: 0.8em; opacity: 0.7;">
                            {move || {
                                let results = state_search.search_results.get();
                                if results.is_empty() {
                                    "".to_string()
                                } else {
                                    format!("{} / {}", state_search.search_current_idx.get() + 1, results.len())
                                }
                            }}
                        </span>
                    </div>
                    <div style="margin-left: auto;">
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
                    <button type="button" on:click=move |e| { e.prevent_default(); save_db_logic(state_c2.clone()); }>"Save"</button>
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
    let container_ref = state.main_container_ref;
    let (scroll_top, set_scroll_top) = RwSignal::new(0.0).split();
    let (viewport_height, set_viewport_height) = RwSignal::new(1000.0).split();

    // Effect to initialize viewport height and handle resize
    Effect::new({
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
        let state = state.clone();
        move || {
            let _ = state.current_bank.get();
            // Only reset if we aren't handling a specific nav target
            if state.nav_target.get_untracked().is_none() {
                if let Some(div) = container_ref.get() {
                    div.set_scroll_top(0);
                    set_scroll_top.set(0.0);
                }
                
                // If we were navigating to a bank (without address), reset the flag
                if state.is_navigating.get_untracked() {
                    let is_navigating = state.is_navigating;
                    leptos::task::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(100).await;
                        is_navigating.set(false);
                    });
                }
            }
        }
    });

    const LINE_HEIGHT: f64 = 20.0;

    let offsets = Memo::new({
        let state = state.clone();
        move |_| {
            let lines = state.disassembly.get();
            let editing = state.editing_block_comment.get();
            let mut current = 0.0;
            let mut off = Vec::with_capacity(lines.len());
            for line in &lines {
                off.push(current);
                let mut height = LINE_HEIGHT;
                
                let is_editing_bc = editing == Some((line.address, line.bank));

                // Block comments add height in all views
                if let Some(ref bc) = line.block_comment {
                    let count = bc.lines().count() as f64;
                    height += count * LINE_HEIGHT;
                } else if is_editing_bc {
                    // New block comment being edited
                    height += LINE_HEIGHT;
                }

                // Symbols only add a separate line height in banked views
                if line.bank != -1 && line.symbol.is_some() {
                    height += LINE_HEIGHT;
                }
                
                current += height;
            }
            (off, current)
        }
    });

    // Effect to handle navigation target
    Effect::new({
        let state = state.clone();
        move || {
            if let Some(target_addr) = state.nav_target.get() {
                let lines = state.disassembly.get();
                if let Some(idx) = lines.iter().position(|l| l.address == target_addr) {
                    let (off, _) = offsets.get();
                    let target_y = off[idx];
                    if let Some(div) = container_ref.get() {
                        div.set_scroll_top(target_y as i32);
                        set_scroll_top.set(target_y);
                        // Small delay to allow scroll events to settle and avoid race with bank reset
                        let is_navigating = state.is_navigating;
                        let nav_target = state.nav_target;
                        leptos::task::spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(100).await;
                            is_navigating.set(false);
                            if nav_target.get_untracked() == Some(target_addr) {
                                nav_target.set(None);
                            }
                        });
                    }
                }
            }
        }
    });

    let range_lines = Memo::new({
        let state = state.clone();
        move |_| {
            let (off, _) = offsets.get();
            if off.is_empty() { return Vec::new(); }

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
            let end = (end_idx + buffer).min(state.disassembly.get().len());
            
            let lines = state.disassembly.get();
            if start >= end || start >= lines.len() {
                return Vec::new();
            }
            lines[start..end].to_vec()
        }
    });

    let on_scroll = {
        let state = state.clone();
        move |ev: web_sys::Event| {
            let div = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
            let top = div.scroll_top() as f64;
            set_scroll_top.set(top);
            set_viewport_height.set(div.client_height() as f64);

            if !state.is_navigating.get_untracked() {
                let (off, _) = offsets.get_untracked();
                let idx = match off.binary_search_by(|v| v.partial_cmp(&top).unwrap()) {
                    Ok(idx) => idx,
                    Err(idx) => idx.saturating_sub(1),
                };
                
                let lines = state.disassembly.get_untracked();
                if let Some(line) = lines.get(idx) {
                    let bank_id = state.current_bank.get_untracked();
                    let bank_hex = format!("{:02X}", bank_id);
                    let addr_hex = format!("{:04X}", line.address);
                    let new_hash = format!("#bank-{}-addr-{}", bank_hex, addr_hex);
                    
                    let window = web_sys::window().unwrap();
                    let history = window.history().unwrap();
                    if window.location().hash().unwrap_or_default() != new_hash {
                        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&new_hash));
                    }
                }
            }
        }
    };

    view! {
        <div 
            class="disassembly-container" 
            node_ref=container_ref 
            on:scroll=on_scroll
            style="position: relative; overflow-y: auto; height: 100%;"
        >
            <div style=move || format!("height: {}px; position: relative;", offsets.get().1)>
                <For
                    each=move || range_lines.get()
                    key=|line| (line.address, line.bank)
                    children={
                        let state = state.clone();
                        let offsets = offsets.clone();
                        move |line| {
                            let line_id = (line.address, line.bank);
                            let line_sig = Signal::derive({
                                let state = state.clone();
                                move || {
                                    state.disassembly.get().iter()
                                        .find(|l| (l.address, l.bank) == line_id)
                                        .cloned()
                                        .unwrap_or(line.clone())
                                }
                            });
                            let top = Signal::derive({
                                let offsets = offsets.clone();
                                let state = state.clone();
                                move || {
                                    let lines = state.disassembly.get();
                                    let (off, _) = offsets.get();
                                    if let Some(idx) = lines.iter().position(|l| (l.address, l.bank) == line_id) {
                                        off[idx]
                                    } else {
                                        0.0
                                    }
                                }
                            });
                            view! { <DisasmRow line=line_sig top=top /> }
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn Highlight(#[prop(into)] text: String, #[prop(into)] query: String, #[prop(into)] active: bool) -> impl IntoView {
    if query.is_empty() || !text.to_lowercase().contains(&query.to_lowercase()) {
        return view! { <span>{text}</span> }.into_any();
    }

    let mut nodes = Vec::new();
    let mut last_end = 0;
    let t_low = text.to_lowercase();
    let q_low = query.to_lowercase();
    
    let highlight_class = if active { "search-highlight-active" } else { "search-highlight" };

    let mut start = 0;
    while let Some(pos) = t_low[start..].find(&q_low) {
        let actual_pos = start + pos;
        let prefix = text[last_end..actual_pos].to_string();
        let match_text = text[actual_pos..actual_pos + query.len()].to_string();
        nodes.push(view! { <span>{prefix}</span> }.into_any());
        nodes.push(view! { <span class=highlight_class>{match_text}</span> }.into_any());
        last_end = actual_pos + query.len();
        start = last_end;
    }
    let suffix = text[last_end..].to_string();
    nodes.push(view! { <span>{suffix}</span> }.into_any());
    nodes.collect_view().into_any()
}

#[component]
fn DisasmRow(#[prop(into)] line: Signal<DisassemblyLine>, #[prop(into)] top: Signal<f64>) -> impl IntoView {
    let state = use_context::<AppState>().expect("state should be provided");

    let is_active_line = Memo::new({
        let state = state.clone();
        move |_| {
            let results = state.search_results.get();
            let idx = state.search_current_idx.get();
            results.get(idx) == Some(&line.get().address)
        }
    });

    let on_symbol_blur = {
        let state = state.clone();
        move |ev: web_sys::FocusEvent| {
            let line = line.get_untracked();
            let val = event_target_inner_text(&ev);
            update_annotation(state.clone(), line.address, line.bank, "symbol", val);
        }
    };

    let on_comment_blur = {
        let state = state.clone();
        move |ev: web_sys::FocusEvent| {
            let line = line.get_untracked();
            let val = event_target_inner_text(&ev);
            update_annotation(state.clone(), line.address, line.bank, "comment", val);
        }
    };

    let on_keydown = {
        let state = state.clone();
        move |ev: web_sys::KeyboardEvent| {
            let key = ev.key();
            let line = line.get_untracked();
            if key == "Enter" {
                ev.prevent_default();
                let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
                let _ = target.blur();
            } else if key == "Escape" {
                ev.prevent_default();
                let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
                let is_symbol = target.class_list().contains("symbol");
                if is_symbol {
                    if line.bank != -1 {
                        target.set_inner_text(&line.symbol.as_ref().map(|s| format!("{}:", s)).unwrap_or_else(|| "???".to_string()));
                    } else {
                        target.set_inner_text(&line.symbol.clone().unwrap_or_else(|| "???".to_string()));
                    }
                } else {
                    target.set_inner_text(&line.comment.as_ref().map(|c| format!("; {}", c)).unwrap_or_default());
                }
                let _ = target.blur();
            }
        }
    };

    let on_block_blur = {
        let state = state.clone();
        move |ev: web_sys::FocusEvent| {
            let line = line.get_untracked();
            state.editing_block_comment.set(None);
            let val = event_target_inner_text(&ev);
            update_annotation(state.clone(), line.address, line.bank, "block_comment", val);
        }
    };

    let on_block_keydown = {
        let state = state.clone();
        move |ev: web_sys::KeyboardEvent| {
            let key = ev.key();
            let ctrl = ev.ctrl_key();
            let line = line.get_untracked();
            if key == "Enter" {
                if ctrl {
                    ev.prevent_default();
                    ev.stop_propagation();
                    
                    let window = web_sys::window().expect("window not found");
                    let document = window.document().expect("document not found").unchecked_into::<web_sys::HtmlDocument>();
                    
                    // execCommand is deprecated but still the most reliable way to handle 
                    // contenteditable newlines while maintaining focus/undo history.
                    #[allow(deprecated)]
                    let _ = document.exec_command("insertLineBreak");

                    // Scroll compensation for the new line
                    if let Some(div) = state.main_container_ref.get() {
                        state.is_navigating.set(true);
                        div.set_scroll_top(div.scroll_top() + 20);
                        
                        let is_nav = state.is_navigating;
                        leptos::task::spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(50).await;
                            is_nav.set(false);
                        });
                    }
                } else {
                    ev.prevent_default();
                    let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
                    let _ = target.blur();
                }
            } else if key == "Escape" {
                ev.prevent_default();
                let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
                target.set_inner_text(&line.block_comment.as_ref().map(|bc| {
                    bc.lines().map(|l| format!("; {}", l)).collect::<Vec<_>>().join("\n")
                }).unwrap_or_default());
                let _ = target.blur();
            }
        }
    };

    let on_operand_blur = {
        let state = state.clone();
        move |ev: web_sys::FocusEvent| {
            let line = line.get_untracked();
            state.editing_operand.set(None);
            if let Some(target_addr) = line.target_address {
                let target_bank = line.target_bank.map(|b| b as i16).unwrap_or(-1);
                let val = event_target_inner_text(&ev);
                update_annotation(state.clone(), target_addr, target_bank, "symbol", val);
            }
        }
    };

    let on_operand_keydown = {
        let state = state.clone();
        move |ev: web_sys::KeyboardEvent| {
            let key = ev.key();
            let line = line.get_untracked();
            if key == "Enter" {
                ev.prevent_default();
                let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
                let _ = target.blur();
            } else if key == "Escape" {
                ev.prevent_default();
                let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
                target.set_inner_text(&line.operand_main.clone());
                let _ = target.blur();
            }
        }
    };

    let on_click_trigger = {
        let state = state.clone();
        move |ev: web_sys::MouseEvent| {
            if ev.shift_key() {
                ev.prevent_default();
                let line = line.get_untracked();
                if line.block_comment.is_none() {
                    // Scroll down by one line height since a block comment just appeared
                    if let Some(div) = state.main_container_ref.get() {
                        div.set_scroll_top(div.scroll_top() + 20);
                    }
                }
                state.editing_block_comment.set(Some((line.address, line.bank)));
            }
        }
    };

    let on_operand_click = {
        let state = state.clone();
        move |ev: web_sys::MouseEvent| {
            if ev.shift_key() {
                ev.prevent_default();
                let line = line.get_untracked();
                if line.target_address.is_some() {
                    state.editing_operand.set(Some((line.address, line.bank)));
                }
            }
        }
    };

    // Effect to focus the block comment field when editing starts
    let bc_ref = NodeRef::<Div>::new();
    Effect::new({
        let state = state.clone();
        move || {
            let line = line.get(); // Subscribe to line updates
            if let Some((addr, bank)) = state.editing_block_comment.get() {
                if addr == line.address && bank == line.bank {
                    if let Some(div) = bc_ref.get() {
                        // Use a small delay to ensure the DOM element is rendered
                        leptos::task::spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(50).await;
                            let _ = div.focus();
                            
                            // Move cursor to the end
                            let window = web_sys::window().unwrap();
                            if let Ok(Some(sel)) = window.get_selection() {
                                sel.select_all_children(&div).unwrap();
                                sel.collapse_to_end().unwrap();
                            }
                        });
                    }
                }
            }
        }
    });

    // Effect to focus the operand field when editing starts
    let op_ref = NodeRef::<leptos::html::Span>::new();
    Effect::new({
        let state = state.clone();
        move || {
            let line = line.get();
            if let Some((addr, bank)) = state.editing_operand.get() {
                if addr == line.address && bank == line.bank {
                    if let Some(div) = op_ref.get() {
                        leptos::task::spawn_local(async move {
                            gloo_timers::future::TimeoutFuture::new(50).await;
                            let _ = div.focus();
                            let window = web_sys::window().unwrap();
                            if let Ok(Some(sel)) = window.get_selection() {
                                sel.select_all_children(&div).unwrap();
                            }
                        });
                    }
                }
            }
        }
    });

    let state_nav = state.clone();
    let on_block_blur_c = on_block_blur.clone();
    let on_block_keydown_c = on_block_keydown.clone();
    let bc_ref_c = bc_ref.clone();
    let on_symbol_blur_c = on_symbol_blur.clone();
    let on_comment_blur_c = on_comment_blur.clone();
    let on_keydown_c = on_keydown.clone();
    let on_click_trigger_c = on_click_trigger.clone();
    let on_operand_click_c = on_operand_click.clone();
    let op_ref_c = op_ref.clone();
    let on_operand_blur_c = on_operand_blur.clone();
    let on_operand_keydown_c = on_operand_keydown.clone();

    view! {
        <div class="grid-row" style=move || format!("position: absolute; top: {}px; width: 100%; display: grid; grid-template-columns: var(--col-addr) var(--col-hex) var(--col-op) var(--col-operand) 1fr;", top.get())>
            {move || {
                let line = line.get();
                let is_editing = state.editing_block_comment.get() == Some((line.address, line.bank));
                let on_block_blur = on_block_blur_c.clone();
                let on_block_keydown = on_block_keydown_c.clone();
                let bc_ref = bc_ref_c.clone();
                let query = state.search_query.get();
                let is_match = !query.is_empty() && line.block_comment.as_ref().map_or(false, |bc| bc.to_lowercase().contains(&query.to_lowercase()));
                let active = is_active_line.get();

                if let Some(ref bc) = line.block_comment {
                    view! {
                        <div class="grid-cell full-width" class:search-match-cell=is_match style="grid-column: 1 / -1;">
                            <div class="comment editable-container" contenteditable="true" node_ref=bc_ref 
                                on:blur=on_block_blur on:keydown=on_block_keydown
                            >
                                <Highlight text={bc.lines().map(|l| format!("; {}", l)).collect::<Vec<_>>().join("\n")} query=query.clone() active=active />
                            </div>
                        </div>
                    }.into_any()
                } else if is_editing {
                    view! {
                        <div class="grid-cell full-width" style="grid-column: 1 / -1;">
                            <div class="comment editable-container" contenteditable="true" node_ref=bc_ref 
                                on:blur=on_block_blur on:keydown=on_block_keydown
                                prop:innerText="; "
                            ></div>
                        </div>
                    }.into_any()
                } else { view! {}.into_any() }
            }}

            {move || {
                let line = line.get();
                let on_symbol_blur = on_symbol_blur_c.clone();
                let on_comment_blur = on_comment_blur_c.clone();
                let on_keydown = on_keydown_c.clone();
                let on_click_trigger = on_click_trigger_c.clone();
                let query = state.search_query.get();
                let active = is_active_line.get();
                
                if line.bank != -1 {
                    let state_nav = state_nav.clone();
                    let on_click_trigger = on_click_trigger.clone();
                    let on_operand_click = on_operand_click_c.clone();
                    let op_ref = op_ref_c.clone();
                    let on_operand_blur = on_operand_blur_c.clone();
                    let on_operand_keydown = on_operand_keydown_c.clone();
                    let is_editing_op = state.editing_operand.get() == Some((line.address, line.bank));
                    
                    let sym_match = !query.is_empty() && line.symbol.as_ref().map_or(false, |s| s.to_lowercase().contains(&query.to_lowercase()));
                    let op_match = !query.is_empty() && line.operand_main.to_lowercase().contains(&query.to_lowercase());
                    let comm_match = !query.is_empty() && line.comment.as_ref().map_or(false, |c| c.to_lowercase().contains(&query.to_lowercase()));

                    view! {
                        {if let Some(ref sym) = line.symbol {
                            let sym_c = sym.clone();
                            let query_c = query.clone();
                            view! {
                                <div class="grid-cell full-width" class:search-match-cell=sym_match style="grid-column: 1 / -1;">
                                    <div class="symbol editable-container" contenteditable="true" 
                                        on:blur=on_symbol_blur on:keydown=on_keydown.clone()
                                    >
                                        <Highlight text={format!("{}:", sym_c)} query=query_c active=active />
                                    </div>
                                </div>
                            }.into_any()
                        } else { view! {}.into_any() }}
                        <div class="grid-cell address" on:click=on_click_trigger.clone()>{line.address_label}</div>
                        <div class="grid-cell hex" on:click=on_click_trigger.clone()>{line.bytes}</div>
                        <div class="grid-cell opcode">{line.opcode}</div>
                        <div class="grid-cell operand" class:search-match-cell=op_match on:click=on_operand_click>
                            <span>{line.operand_prefix}</span>
                            {if is_editing_op {
                                view! {
                                    <span class="symbol editable-container" contenteditable="true" 
                                          node_ref=op_ref on:blur=on_operand_blur on:keydown=on_operand_keydown
                                          prop:innerText={line.operand_main.clone()}
                                    ></span>
                                }.into_any()
                            } else if let Some(target_addr) = line.target_address {
                                let target_bank = line.target_bank;
                                let state_nav = state_nav.clone();
                                let is_symbol = line.operand_is_symbol;
                                let query_c = query.clone();
                                view! {
                                    <a href="#" 
                                       class:symbol=is_symbol
                                       on:click=move |e| { 
                                           e.prevent_default(); 
                                           if !e.shift_key() {
                                               navigate(state_nav.clone(), target_bank, target_addr); 
                                           }
                                       }>
                                        <Highlight text=line.operand_main.clone() query=query_c active=active />
                                    </a>
                                }.into_any()
                            } else {
                                let query_c = query.clone();
                                view! { <Highlight text=line.operand_main.clone() query=query_c active=active /> }.into_any()
                            }}
                            <span>{line.operand_suffix}</span>
                        </div>
                        <div class="grid-cell comment-cell" class:search-match-cell=comm_match on:click=on_click_trigger.clone()>
                            <div class="comment editable-container" contenteditable="true" 
                                on:blur=on_comment_blur on:keydown=on_keydown.clone()
                            >
                                <Highlight text={line.comment.as_ref().map(|c| format!("; {}", c)).unwrap_or_default()} query=query.clone() active=active />
                            </div>
                        </div>
                    }.into_any()
                } else {
                    // Global Equate
                    let on_click_trigger = on_click_trigger.clone();
                    let sym_val = line.symbol.clone().unwrap_or_else(|| "???".to_string());
                    let comm_val = line.comment.as_ref().map(|c| format!("; {}", c)).unwrap_or_default();
                    let sym_match = !query.is_empty() && sym_val.to_lowercase().contains(&query.to_lowercase());
                    let comm_match = !query.is_empty() && comm_val.to_lowercase().contains(&query.to_lowercase());

                    view! {
                        <div class="grid-cell address" class:search-match-cell=sym_match style="grid-column: 1 / span 4; display: flex; align-items: baseline;" on:click=on_click_trigger.clone()>
                            <div class="symbol editable-container" contenteditable="true" 
                                on:blur=on_symbol_blur on:keydown=on_keydown.clone()
                            >
                                <Highlight text=sym_val query=query.clone() active=active />
                            </div>
                            <span style="margin-left: 8px;">" = " {line.address_label.clone()}</span>
                        </div>
                        <div class="grid-cell comment-cell" class:search-match-cell=comm_match on:click=on_click_trigger.clone()>
                            <div class="comment editable-container" contenteditable="true" 
                                on:blur=on_comment_blur on:keydown=on_keydown.clone()
                            >
                                <Highlight text=comm_val query=query.clone() active=active />
                            </div>
                        </div>
                    }.into_any()
                }
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
        let lines = text.lines().map(|line| {
            let mut l = line.trim_start();
            if l.starts_with(';') {
                l = &l[1..];
                if l.starts_with(' ') { l = &l[1..]; }
            }
            l
        }).collect::<Vec<_>>().join("\n").trim_end().to_string();
        return lines;
    }
    text.to_string()
}

fn navigate(state: AppState, target_bank: Option<u8>, target_address: u16) {
    let bank_id = target_bank.unwrap_or(255);
    
    // Update state
    state.is_navigating.set(true);
    state.current_bank.set(bank_id);
    state.nav_target.set(Some(target_address));

    // Update URL hash to create history entry
    let window = web_sys::window().unwrap();
    let bank_hex = format!("{:02X}", bank_id);
    let addr_hex = format!("{:04X}", target_address);
    let hash = format!("#bank-{}-addr-{}", bank_hex, addr_hex);
    window.location().set_hash(&hash).unwrap();
}

fn event_target_inner_text(ev: &web_sys::FocusEvent) -> String {
    let target = ev.target().unwrap().unchecked_into::<web_sys::HtmlElement>();
    target.inner_text()
}
