// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    widget::{self, settings, toggler},
    Apply, Element,
};

use cosmic_config::{ConfigGet, ConfigSet};
use cosmic_settings_config::{shortcuts, Action, Binding, Shortcuts};
use cosmic_settings_page::Section;
use cosmic_settings_page::{self as page, section};
use slab::Slab;
use slotmap::SlotMap;

#[derive(Copy, Clone, Debug)]
pub enum Message {
    SuperKey(usize),
}

pub struct Page {
    pub super_key_selections: Vec<String>,
    pub super_key_active: Option<usize>,
}

impl Default for Page {
    fn default() -> Self {
        Page {
            super_key_selections: vec![
                fl!("super-key", "launcher"),
                fl!("super-key", "workspaces"),
                fl!("super-key", "applications"),
            ],
            super_key_active: super_key_active_config(),
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SuperKey(id) => {
                let action = match id {
                    0 => shortcuts::action::System::Launcher,
                    1 => shortcuts::action::System::WorkspaceOverview,
                    2 => shortcuts::action::System::AppLibrary,
                    _ => return,
                };

                self.super_key_active = Some(id);
                super_key_set(action);
            }
        }
    }
}

impl page::Page<crate::pages::Message> for Page {
    #[allow(clippy::too_many_lines)]
    fn content(
        &self,
        sections: &mut SlotMap<section::Entity, Section<crate::pages::Message>>,
    ) -> Option<page::Content> {
        Some(vec![
            sections.insert(super_key_action()),
            sections.insert(window_controls()),
        ])
    }

    fn info(&self) -> page::Info {
        page::Info::new(
            "window-management",
            "preferences-window-management-symbolic",
        )
        .title(fl!("window-management"))
        .description(fl!("window-management", "desc"))
    }
}

impl page::AutoBind<crate::pages::Message> for Page {}

pub fn super_key_action() -> Section<crate::pages::Message> {
    let mut descriptions = Slab::new();

    let super_key = descriptions.insert(fl!("super-key"));
    let _launcher = descriptions.insert(fl!("super-key", "launcher"));
    let _workspaces = descriptions.insert(fl!("super-key", "workspaces"));
    let _applications = descriptions.insert(fl!("super-key", "applications"));

    Section::default()
        .descriptions(descriptions)
        .view::<Page>(move |_binder, page, section| {
            let descriptions = &section.descriptions;

            settings::view_section(&section.title)
                .add(
                    settings::item::builder(&descriptions[super_key]).control(widget::dropdown(
                        &page.super_key_selections,
                        page.super_key_active,
                        Message::SuperKey,
                    )),
                )
                .apply(Element::from)
                .map(crate::pages::Message::WindowManagement)
        })
}

pub fn window_controls() -> Section<crate::pages::Message> {
    let mut descriptions = Slab::new();

    let maximize = descriptions.insert(fl!("window-controls", "maximize"));
    let minimize = descriptions.insert(fl!("window-controls", "minimize"));

    Section::default()
        .title(fl!("window-controls"))
        .descriptions(descriptions)
        .view::<Page>(move |binder, _page, section| {
            let desktop = binder
                .page::<super::Page>()
                .expect("desktop page not found");
            let descriptions = &section.descriptions;

            settings::view_section(&section.title)
                .add(settings::item(
                    &descriptions[maximize],
                    toggler(
                        None,
                        desktop.cosmic_tk.show_maximize,
                        super::Message::ShowMaximizeButton,
                    ),
                ))
                .add(settings::item(
                    &descriptions[minimize],
                    toggler(
                        None,
                        desktop.cosmic_tk.show_minimize,
                        super::Message::ShowMinimizeButton,
                    ),
                ))
                .apply(Element::from)
                .map(crate::pages::Message::Desktop)
        })
}

fn super_key_active_config() -> Option<usize> {
    let super_binding = Binding::new(shortcuts::Modifiers::new().logo(), None);

    let config = shortcuts::context().ok()?;
    let shortcuts = shortcuts::shortcuts(&config);

    let new_id = shortcuts
        .iter()
        .find(|(binding, _action)| binding == &&super_binding)
        .and_then(|(_, action)| match action {
            Action::System(shortcuts::action::System::Launcher) => Some(0),
            Action::System(shortcuts::action::System::WorkspaceOverview) => Some(1),
            Action::System(shortcuts::action::System::AppLibrary) => Some(2),
            _ => None,
        });

    new_id
}

fn super_key_set(action: shortcuts::action::System) {
    let Ok(config) = shortcuts::context() else {
        return;
    };

    let mut shortcuts = config.get::<Shortcuts>("custom").unwrap_or_default();

    shortcuts.0.insert(
        Binding::new(shortcuts::Modifiers::new().logo(), None),
        Action::System(action),
    );

    _ = config.set("custom", &shortcuts);
}
