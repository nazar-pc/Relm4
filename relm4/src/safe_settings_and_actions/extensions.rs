/*
 * SPDX-FileCopyrightText: 2023 Eduardo Javier Alvarado Aarón <eduardo.javier.alvarado.aaron@gmail.com>
 *
 * SPDX-License-Identifier: (Apache-2.0 or MIT)
 */

//! Traits that extend types that use settings or actions to make them compatible with
//! the types generated by [`safe_settings_and_actions!`](crate::safe_settings_and_actions!).

use super::*;
use gio::prelude::{ActionExt, IsA, SettingsExt, SettingsExtManual, ToVariant};
use gtk::{gio, glib};

/// Trait that extends [`gio::Action`] with action safety methods.
#[allow(unused_qualifications)]
pub trait SafeAction: gio::prelude::ActionExt {
    /// Safe version of [`state`](gio::prelude::ActionExt::state) for stateful action safeties.
    fn state_safe<'a, T: ActionSafety + Stateful<'a>>(&self, _safety: T) -> T::Owned {
        self.state().unwrap().get().unwrap()
    }

    /// Safe version of [`state`](gio::prelude::ActionExt::state) for action safeties with variants.
    fn state_safe_enum<T: for<'a> WithValue<'a> + DetailableAction>(&self) -> T {
        T::from_variant(&self.state().unwrap())
    }

    /// Safe version of [`connect_state_notify`](gio::prelude::ActionExt::connect_state_notify) for stateful action safeties.
    fn connect_state_notify_safe<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> Stateful<'a>,
        F: Fn(T, &Self, <T as Stateful<'_>>::Mapping) + 'static,
    {
        self.connect_state_notify(move |this| {
            callback(T::SELF, this, T::map(&this.state().unwrap()))
        })
    }

    /// Safe version of [`connect_state_notify`](gio::prelude::ActionExt::connect_state_notify) for action safeties with variants.
    fn connect_state_notify_safe_enum<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: for<'a> WithValue<'a> + DetailableAction,
        F: Fn(&Self, T) + 'static,
    {
        self.connect_state_notify(move |this| {
            callback(this, T::from_variant(&this.state().unwrap()))
        })
    }
}

impl<T: IsA<gio::Action>> SafeAction for T {}

/// Trait that extends [`gtk::Actionable`] with action safety methods.
pub trait SafeActionable: gtk::prelude::ActionableExt {
    /// Appropriately assigns the [name][n] and [target value][t] of an action
    /// according to an action safety with variants or without value nor variants.
    ///
    /// [n]: gtk::prelude::ActionableExt::set_action_name
    /// [t]: gtk::prelude::ActionableExt::set_action_target_value
    fn set_action_safe<T: DetailableAction>(&self, safety: T) {
        // self.set_detailed_action_name(safety.detailed_action_name());
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(safety.to_variant().as_ref());
    }

    /// Appropriately assigns the [name][n] and [target value][t] of an
    /// action according to an action safety with value and without variants.
    ///
    /// [n]: gtk::prelude::ActionableExt::set_action_name
    /// [t]: gtk::prelude::ActionableExt::set_action_target_value
    fn set_target_safe<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value)
    where
        T: ActionSafety + NotDetailable,
    {
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(Some(&target.to_variant()))
    }
}

impl<T: IsA<gtk::Actionable>> SafeActionable for T {}

/// Trait that extends [`gio::ActionMap`] with action safety methods.
pub trait SafeActionMap: gio::prelude::ActionMapExt {
    /// Safe version of [`lookup_action`](gio::prelude::ActionMapExt::lookup_action) for action safeties.
    fn lookup_action_safe<T: ActionSafety>(&self, _safety: T) -> Option<gio::Action> {
        self.lookup_action(T::NAME)
    }

    /// Safe version of [`remove_action`](gio::prelude::ActionMapExt::remove_action) for action safeties.
    fn remove_action_safe<T: ActionSafety>(&self, _safety: T) {
        self.remove_action(T::NAME)
    }
}

impl<T: IsA<gio::ActionMap>> SafeActionMap for T {}

/// Trait that extends [`gtk::Application`] with action safety methods.
pub trait SafeApplication: gtk::prelude::GtkApplicationExt {
    /// Safe version of [`accels_for_action`](gtk::prelude::GtkApplicationExt::accels_for_action)
    /// for action safeties with variants or without value nor variants.
    fn accels_for_action_safe<T: DetailableAction>(&self, safety: T) -> Vec<glib::GString> {
        self.accels_for_action(safety.detailed_action_name())
    }

    /// Safe version of [`set_accels_for_action`](gtk::prelude::GtkApplicationExt::set_accels_for_action)
    /// for action safeties with variants or without value nor variants.
    fn set_accels_for_action_safe<T: DetailableAction>(&self, safety: T, accels: &[&str]) {
        self.set_accels_for_action(safety.detailed_action_name(), accels)
    }
}

impl<T: IsA<gtk::Application>> SafeApplication for T {}

#[cfg(feature = "macros")]
/// Trait that extends [`gio::Menu`] with methods compatible with [`relm4_macros::view!`] and action safety methods.
pub trait RelmMenu: IsA<gio::Menu> {
    /// Adds a menu item for an action safety with variants or without value nor variants.
    fn action<T: DetailableAction>(&self, safety: T, label: &str);

    /// Adds a menu item for an action safety with value and without variants.
    fn target<'a, T: WithValue<'a>>(&self, safety: T, target: T::Value, label: &str)
    where
        T: ActionSafety + NotDetailable;

    /// Adds a menu item for a detailed action name.
    fn detailed(&self, action: &str, label: &str);

    /// Adds a section.
    fn section(&self, model: &impl IsA<gio::MenuModel>, label: &str);

    /// Adds a submenu.
    fn submenu(&self, model: &impl IsA<gio::MenuModel>, label: &str);

    /// Adds a placeholder for a widget.
    fn widget(&self, label: &str);
}

#[cfg(feature = "macros")]
impl RelmMenu for gio::Menu {
    fn action<T: DetailableAction>(&self, safety: T, label: &str) {
        // self.append(Some(label), Some(safety.detailed_action_name()));
        let item = gio::MenuItem::new(Some(label), None);
        item.set_action_and_target_value(Some(T::FULL_NAME), safety.to_variant().as_ref());
        self.append_item(&item);
    }

    fn target<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value, label: &str)
    where
        T: ActionSafety,
    {
        let item = gio::MenuItem::new(Some(label), None);
        item.set_action_and_target_value(Some(T::FULL_NAME), Some(&target.to_variant()));
        self.append_item(&item);
    }

    fn detailed(&self, action: &str, label: &str) {
        self.append(Some(label), Some(action));
    }

    fn section(&self, model: &impl IsA<gio::MenuModel>, label: &str) {
        self.append_section((!label.is_empty()).then_some(label), model);
    }

    fn submenu(&self, model: &impl IsA<gio::MenuModel>, label: &str) {
        self.append_submenu(Some(label), model);
    }

    fn widget(&self, name: &str) {
        let item = gio::MenuItem::new(None, None);
        item.set_attribute_value("custom", Some(&name.to_variant()));
        self.append_item(&item);
    }
}

/// Trait that extends [`gio::Settings`] with action safety methods.
pub trait SafeSettings: IsA<gio::Settings> {
    /// Safe version of [`create_action`](gio::prelude::SettingsExt::create_action)
    /// for action safeties with variants or without value nor variants.
    fn create_action_safe<T: DetailableAction>(&self) -> gio::Action {
        self.create_action(T::NAME)
    }

    /// Safe version of [`bind`](gio::prelude::SettingsExtManual::bind)
    /// for setting safeties with variants or without value nor variants.
    fn bind_safe<'a, T: DetailableSetting>(
        &'a self,
        object: &'a impl IsA<glib::Object>,
        property: &'a str,
    ) -> gio::BindingBuilder<'a> {
        self.bind(T::NAME, object, property)
    }

    /// Safe version of [`set`](gio::prelude::SettingsExtManual::set) for stateful setting safeties without value.
    fn set_safe<'a, T: WithoutValue + Stateful<'a>>(
        &self,
        _safety: T,
        state: T::State,
    ) -> Result<(), glib::BoolError> {
        self.set(T::NAME, state.to_variant())
    }

    /// Safe version of [`set_value`](gio::prelude::SettingsExt::set_value) for setting safeties with variants.
    fn set_safe_enum<T>(&self, safety: T) -> Result<(), glib::BoolError>
    where
        T: for<'a> WithValue<'a> + DetailableSetting,
    {
        self.set_value(T::NAME, &safety.to_variant().unwrap()) // NOTE could be unwrap_unchecked()
    }

    /// Safe version of [`get`](gio::prelude::SettingsExtManual::get) for stateful setting safeties without value.
    fn get_safe<'a, T: WithoutValue + Stateful<'a>>(&self, _safety: T) -> T::Owned {
        self.get(T::NAME)
    }

    /// Safe version of [`value`](gio::prelude::SettingsExt::value) for setting safeties with variants.
    fn get_safe_enum<T: for<'a> WithValue<'a> + DetailableSetting>(&self) -> T {
        T::from_variant(&self.value(T::NAME))
    }
}

impl<T: IsA<gio::Settings>> SafeSettings for T {}

/// Trait that extends [`gio::SimpleAction`] with action safety methods.
pub trait SafeSimpleAction: IsA<gio::SimpleAction> {
    /// Safe version of [`new`](gio::SimpleAction::new) for stateless action safeties.
    fn new_safe<T: ActionSafety + Stateless>() -> Self;

    /// Safe version of [`new_stateful`](gio::SimpleAction::new_stateful) for stateful action safeties.
    fn new_stateful_safe<'a, T: ActionSafety + Stateful<'a>>(state: T::State) -> Self;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate) for stateless action safeties without value.
    fn connect_activate_safe<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + WithoutValue + Stateless,
        F: Fn(T, &Self) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for stateless action safeties with value and without variants.
    fn connect_activate_safe_with_target<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> WithValue<'a> + Stateless + NotDetailable,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for stateful action safeties without value.
    fn connect_activate_safe_with_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + WithoutValue + for<'a> Stateful<'a>,
        F: Fn(T, &Self, <T as Stateful<'_>>::Mapping) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// to conveniently mutate state for stateful action safeties without value.
    fn connect_activate_safe_with_mut_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + WithoutValue + for<'a> Stateful<'a>,
        F: Fn(T, &Self, &mut <T as Stateful<'_>>::Owned) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for stateful action safeties with value and without variants.
    fn connect_activate_safe_with_target_and_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a> + NotDetailable,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, <T as Stateful<'_>>::Mapping) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// to conveniently mutate state for stateful action safeties with value and without variants.
    fn connect_activate_safe_with_target_and_mut_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a> + NotDetailable,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, &mut <T as Stateful<'_>>::Owned) + 'static;

    /// Safe version of [`connect_activate`](gio::SimpleAction::connect_activate)
    /// for action safeties with variants.
    fn connect_activate_safe_enum<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> WithValue<'a> + DetailableAction,
        F: Fn(&Self, T) + 'static;
}

impl SafeSimpleAction for gio::SimpleAction {
    fn new_safe<T: ActionSafety>() -> Self {
        gio::SimpleAction::new(T::NAME, T::variant_type().as_deref())
    }

    fn new_stateful_safe<'a, T: Stateful<'a>>(state: T::State) -> Self {
        gio::SimpleAction::new_stateful(T::NAME, T::variant_type().as_deref(), state.to_variant())
    }

    fn connect_activate_safe<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety,
        F: Fn(T, &Self) + 'static,
    {
        self.connect_activate(move |this, _| callback(T::SELF, this))
    }

    fn connect_activate_safe_with_target<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> WithValue<'a>,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping) + 'static,
    {
        self.connect_activate(move |this, variant| {
            callback(T::SELF, this, T::map(variant.unwrap()))
        })
    }

    fn connect_activate_safe_with_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> Stateful<'a>,
        F: Fn(T, &Self, <T as Stateful<'_>>::Mapping) + 'static,
    {
        self.connect_activate(move |this, _| {
            callback(T::SELF, this, T::map(&this.state().unwrap()))
        })
    }

    fn connect_activate_safe_with_mut_state<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: ActionSafety + for<'a> Stateful<'a>,
        F: Fn(T, &Self, &mut <T as Stateful<'_>>::Owned) + 'static,
    {
        self.connect_activate(move |this, _| {
            let mut state = this.state().unwrap().get().unwrap();
            callback(T::SELF, this, &mut state);
            this.set_state(state.to_variant());
        })
    }

    fn connect_activate_safe_with_target_and_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a>,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, <T as Stateful<'_>>::Mapping) + 'static,
    {
        self.connect_activate(move |this, variant| {
            callback(
                T::SELF,
                this,
                <T as WithValue>::map(variant.unwrap()),
                <T as Stateful>::map(&this.state().unwrap()),
            )
        })
    }

    fn connect_activate_safe_with_target_and_mut_state<T, F>(
        &self,
        callback: F,
    ) -> glib::SignalHandlerId
    where
        for<'a> T: ActionSafety + WithValue<'a> + Stateful<'a>,
        F: Fn(T, &Self, <T as WithValue<'_>>::Mapping, &mut <T as Stateful<'_>>::Owned) + 'static,
    {
        self.connect_activate(move |this, variant| {
            let mut state = this.state().unwrap().get().unwrap();
            callback(
                T::SELF,
                this,
                <T as WithValue>::map(variant.unwrap()),
                &mut state,
            );
            this.set_state(state.to_variant());
        })
    }

    fn connect_activate_safe_enum<T, F>(&self, callback: F) -> glib::SignalHandlerId
    where
        T: for<'a> WithValue<'a> + DetailableAction,
        F: Fn(&Self, T) + 'static,
    {
        self.connect_activate(move |this, variant| {
            callback(this, T::from_variant(variant.unwrap()))
        })
    }
}

#[cfg(feature = "libadwaita")]
/// Trait that extends [`adw::Toast`] with action safety methods.
pub trait SafeToast {
    /// Appropriately assigns the [name][n] and [target value][t] of an action
    /// according to an action safety with variants or without value nor variants.
    ///
    /// [n]: adw::Toast::set_action_name
    /// [t]: adw::Toast::set_action_target_value
    fn set_action_safe<T: DetailableAction>(&self, safety: T);

    /// Appropriately assigns the [name][n] and [target value][t] of an
    /// action according to an action safety with value and without variants.
    ///
    /// [n]: adw::Toast::set_action_name
    /// [t]: adw::Toast::set_action_target_value
    fn set_target_safe<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value)
    where
        T: ActionSafety + NotDetailable;
}

#[cfg(feature = "libadwaita")]
impl SafeToast for adw::Toast {
    fn set_action_safe<T: DetailableAction>(&self, safety: T) {
        // self.set_detailed_action_name(safety.detailed_action_name());
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(safety.to_variant().as_ref());
    }

    fn set_target_safe<'a, T: WithValue<'a>>(&self, _safety: T, target: T::Value)
    where
        T: ActionSafety + NotDetailable,
    {
        self.set_action_name(Some(T::FULL_NAME));
        self.set_action_target_value(Some(&target.to_variant()))
    }
}
