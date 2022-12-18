// Copyright 2022 Doug Powers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use druid::debug_state::DebugState;
use druid::keyboard_types::Key;
use druid::widget::prelude::*;
use druid::widget::{Label, LabelText};
use druid::{theme, Modifiers, Affine, Data, Insets, LinearGradient, UnitPoint, RawMods, MouseButton, Color};
use tracing::{instrument, trace};
use crate::vmconfig::{VMConfigVersion4, VMColor};

const LABEL_INSETS: Insets = Insets::uniform_xy(8., 2.);

pub struct VMButton<T> {
    label: Label<T>,
    label_size: Size,
    action: Box<dyn Fn(&mut EventCtx)>,
    light_color: Color,
    dark_color: Color,
    disabled_light_color: Color,
    disabled_dark_color: Color,
}

impl<T: Data> VMButton<T> {
    // pub fn new(text: impl Into<LabelText<T>>) -> VMButton<T> {
    pub fn new(config: &VMConfigVersion4, text: impl Into<LabelText<T>>, action: impl Fn(&mut EventCtx) + 'static, is_alert: bool) -> VMButton<T> {
        // VMButton::from_label(Label::new(text))
        VMButton::from_label(config, Label::new(text), action, is_alert)
    }

    // pub fn from_label(label: Label<T>) -> VMButton<T> {
    pub fn from_label(config: &VMConfigVersion4, label: Label<T>, action: impl Fn(&mut EventCtx) + 'static, is_alert: bool) -> VMButton<T> {
        VMButton {
            label,
            label_size: Size::ZERO,
            action: Box::new(action),
            light_color: if is_alert {
                config.get_color(VMColor::AlertButtonLight).expect("Couldn't find light alert button color")
            } else {
                config.get_color(VMColor::ButtonLight).expect("Couldn't find light button color")
            },
            dark_color: if is_alert {
                config.get_color(VMColor::AlertButtonDark).expect("Couldn't find dark alert button color")
            } else {
                config.get_color(VMColor::ButtonDark).expect("Couldn't find dark button color")
            },
            disabled_light_color: config.get_color(VMColor::DisabledButtonLight).expect("Couldn't find light disabled button color"),
            disabled_dark_color: config.get_color(VMColor::DisabledButtonDark).expect("Couldn't find dark disabled button color"),
        }
    }
}

impl<T: Data> Widget<T> for VMButton<T> {
    #[instrument(name = "Button", level = "trace", skip(self, ctx, event, _data, _env))]
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(mouse_event) => {
                if mouse_event.button == MouseButton::Left && !ctx.is_disabled() {
                    ctx.set_active(true);
                    ctx.request_paint();
                    trace!("Widget {:?} pressed", ctx.widget_id());
                }
            }
            Event::MouseUp(mouse_event) => {
                if ctx.is_active() && mouse_event.button == MouseButton::Left {
                    ctx.set_active(false);
                    if ctx.is_hot() && !ctx.is_disabled() {
                        (self.action)(ctx);
                    }
                    ctx.request_paint();
                    trace!("Widget {:?} released", ctx.widget_id());
                }
            }
            Event::KeyDown(key_event) => {
                let mut key_event = key_event.clone();
                key_event.mods.set(Modifiers::NUM_LOCK, false);
                key_event.mods.set(Modifiers::SCROLL_LOCK, false);
                key_event.mods.set(Modifiers::CAPS_LOCK, false);
                match &key_event.key {
                    Key::Character(char) if char == " " && key_event.mods == RawMods::None  => {
                        (self.action)(ctx);
                    },
                    Key::Enter if key_event.mods == RawMods::None => {
                        (self.action)(ctx);
                    }
                    _ => () 
                }
            }
            _ => (),
        }
    }

    #[instrument(name = "Button", level = "trace", skip(self, ctx, event, data, env))]
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
            ctx.request_paint();
        }
        if let druid::LifeCycle::BuildFocusChain = event {
            ctx.register_for_focus();
        }
        self.label.lifecycle(ctx, event, data, env)
    }

    #[instrument(name = "Button", level = "trace", skip(self, ctx, old_data, data, env))]
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.label.update(ctx, old_data, data, env)
    }

    #[instrument(name = "Button", level = "trace", skip(self, ctx, bc, data, env))]
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("Button");
        let padding = Size::new(LABEL_INSETS.x_value(), LABEL_INSETS.y_value());
        let label_bc = bc.shrink(padding).loosen();
        self.label_size = self.label.layout(ctx, &label_bc, data, env);
        // HACK: to make sure we look okay at default sizes when beside a textbox,
        // we make sure we will have at least the same height as the default textbox.
        let min_height = env.get(theme::BORDERED_WIDGET_HEIGHT);
        let baseline = self.label.baseline_offset();
        ctx.set_baseline_offset(baseline + LABEL_INSETS.y1);

        let button_size = bc.constrain(Size::new(
            self.label_size.width + padding.width,
            (self.label_size.height + padding.height).max(min_height),
        ));
        trace!("Computed button size: {}", button_size);
        button_size
    }

    #[instrument(name = "Button", level = "trace", skip(self, ctx, data, env))]
    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let is_active = ctx.is_active() && !ctx.is_disabled();
        let is_hot = ctx.is_hot();
        let is_focused = ctx.is_focused();
        let size = ctx.size();
        let stroke_width = env.get(theme::BUTTON_BORDER_WIDTH);

        let rounded_rect = size
            .to_rect()
            .inset(-stroke_width / 2.0)
            .to_rounded_rect(env.get(theme::BUTTON_BORDER_RADIUS));

        let bg_gradient = if ctx.is_disabled() {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    // env.get(theme::DISABLED_BUTTON_LIGHT),
                    // env.get(theme::DISABLED_BUTTON_DARK),
                    self.disabled_light_color.clone(),
                    self.disabled_dark_color.clone(),
                ),
            )
        } else if is_active {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                // (env.get(theme::BUTTON_DARK), env.get(theme::BUTTON_LIGHT)),
                (
                    self.dark_color.clone(),
                    self.light_color.clone(),
                ),
            )
        } else {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                // (env.get(theme::BUTTON_LIGHT), env.get(theme::BUTTON_DARK)),
                (
                    self.light_color.clone(),
                    self.dark_color.clone(),
                )
            )
        };

        let border_color = if (is_hot || is_focused) && !ctx.is_disabled() {
            env.get(theme::BORDER_LIGHT)
        } else {
            env.get(theme::BORDER_DARK)
        };

        ctx.stroke(rounded_rect, &border_color, stroke_width);

        ctx.fill(rounded_rect, &bg_gradient);

        let label_offset = (size.to_vec2() - self.label_size.to_vec2()) / 2.0;

        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(label_offset));
            self.label.paint(ctx, data, env);
        });
    }

    fn debug_state(&self, _data: &T) -> DebugState {
        DebugState {
            display_name: self.short_type_name().to_string(),
            main_value: self.label.text().to_string(),
            ..Default::default()
        }
    }
}