//! macOS 屏幕顶部菜单栏里的快捷项。

use std::sync::{Mutex, OnceLock};

use eframe::egui;
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::models::Variable;

static CONTEXT: OnceLock<egui::Context> = OnceLock::new();
static ACTIONS: Mutex<Vec<Action>> = Mutex::new(Vec::new());

const HEADER_ITEMS: usize = 3;

#[derive(Clone, Copy)]
pub enum Action {
    Open,
    CopyAll,
    CopyOne(i64),
    Edit(i64),
    Quit,
}

pub struct Bar {
    tray: TrayIcon,
    menu: Menu,
}

pub fn install(ctx: &egui::Context) -> Option<Bar> {
    let _ = CONTEXT.set(ctx.clone());
    let menu = Menu::new();
    let open = MenuItem::with_id("open", "Open key-desk", true, None);
    let copy = MenuItem::with_id("copy-env", "Copy .env", true, None);
    let separator = PredefinedMenuItem::separator();
    if menu.append(&open).is_err()
        || menu.append(&copy).is_err()
        || menu.append(&separator).is_err()
    {
        log::warn!("menu bar items could not be added");
        return None;
    }

    let _ = MenuEvent::set_event_handler(Some(|event: MenuEvent| {
        let Some(action) = parse_action(&event.id.0) else {
            return;
        };
        if let Ok(mut actions) = ACTIONS.lock() {
            actions.push(action);
        }
        if let Some(ctx) = CONTEXT.get() {
            ctx.request_repaint();
        }
    }));

    let icon = mark_icon()?;
    match TrayIconBuilder::new()
        .with_menu(Box::new(menu.clone()))
        .with_icon(icon)
        .with_icon_as_template(true)
        .with_tooltip("key-desk")
        .with_title("KD")
        .build()
    {
        Ok(tray) => {
            log::info!("menu bar shortcut ready");
            Some(Bar { tray, menu })
        }
        Err(err) => {
            log::warn!("menu bar shortcut failed: {err}");
            None
        }
    }
}

/// `None` means the vault is still locked. `Some` is the current stored list.
pub fn sync(bar: &Bar, variables: Option<&[Variable]>) {
    let _ = &bar.tray;
    while bar.menu.items().len() > HEADER_ITEMS {
        if bar.menu.remove_at(HEADER_ITEMS).is_none() {
            break;
        }
    }

    match variables {
        None => {
            let locked = MenuItem::with_id("locked", "Unlock to see variables", false, None);
            let _ = bar.menu.append(&locked);
        }
        Some([]) => {
            let empty = MenuItem::with_id("empty", "No variables", false, None);
            let _ = bar.menu.append(&empty);
        }
        Some(variables) => {
            for variable in variables {
                let label = menu_label(&variable.key, &variable.scope);
                let row = Submenu::with_id(format!("row-{}", variable.id), label, true);
                let copy = MenuItem::with_id(format!("copy:{}", variable.id), "Copy", true, None);
                let edit = MenuItem::with_id(format!("edit:{}", variable.id), "Edit", true, None);
                let _ = row.append(&copy);
                let _ = row.append(&edit);
                let _ = bar.menu.append(&row);
            }
        }
    }

    let _ = bar.menu.append(&PredefinedMenuItem::separator());
    let quit = MenuItem::with_id("quit", "Quit", true, None);
    let _ = bar.menu.append(&quit);
}

pub fn take_actions() -> Vec<Action> {
    ACTIONS
        .lock()
        .map(|mut actions| std::mem::take(&mut *actions))
        .unwrap_or_default()
}

fn parse_action(id: &str) -> Option<Action> {
    if let Some(rest) = id.strip_prefix("copy:") {
        return rest.parse().ok().map(Action::CopyOne);
    }
    if let Some(rest) = id.strip_prefix("edit:") {
        return rest.parse().ok().map(Action::Edit);
    }
    match id {
        "open" => Some(Action::Open),
        "copy-env" => Some(Action::CopyAll),
        "quit" => Some(Action::Quit),
        _ => None,
    }
}

fn menu_label(key: &str, scope: &str) -> String {
    let key = key.replace('&', "&&");
    let scope = scope.replace('&', "&&");
    if scope.is_empty() {
        key
    } else {
        format!("{key}    {scope}")
    }
}

fn mark_icon() -> Option<Icon> {
    const N: u32 = 16;
    let mut rgba = vec![0u8; (N * N * 4) as usize];
    let mut set = |x: u32, y: u32| {
        if x >= N || y >= N {
            return;
        }
        let index = ((y * N + x) * 4) as usize;
        rgba[index + 3] = 255;
    };
    for y in 3..10 {
        for x in 2..9 {
            let dx = x as i32 - 5;
            let dy = y as i32 - 6;
            let distance = dx * dx + dy * dy;
            if (8..=13).contains(&distance) {
                set(x, y);
            }
        }
    }
    for x in 8..14 {
        set(x, 6);
        set(x, 7);
    }
    set(12, 8);
    set(13, 8);
    set(12, 9);
    Icon::from_rgba(rgba, N, N).ok()
}
