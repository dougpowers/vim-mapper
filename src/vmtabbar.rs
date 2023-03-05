use druid::{Size, Widget, piet::{PietTextLayout, Text, TextLayoutBuilder, TextLayout},Color, RenderContext, Rect, Point, FontFamily, Affine, Vec2};

use crate::{vmconfig::{VMConfigVersion4, VMColor}};
use crate::constants::*;

pub struct VMTabBar {
    // tab_names: Vec<String>,
    // tab_label_layouts: Vec<Option<PietTextLayout>>,
    // tab_label_sizes: Vec<Option<Size>>,
    tabs: Vec<(String, Option<PietTextLayout>, Option<Size>)>,
    active_tab: usize,
    tab_text_color: Color,
    tab_background_color_active: Color,
    tab_background_color_inactive: Color,
    tab_active_indicator_color: Color,
}

impl VMTabBar {
    pub fn new(config: &VMConfigVersion4, tab_names: &Vec<String>, active_tab: usize) -> VMTabBar {
        let mut tabs: Vec<(String, Option<PietTextLayout>, Option<Size>)> = vec![];
        for name in tab_names {
            tabs.push((
                name.clone(),
                None,
                None,
            ));
        }
        VMTabBar {
            tabs,
            active_tab,
            tab_text_color: config.get_color(VMColor::LabelTextColor).unwrap(),
            tab_background_color_active: config.get_color(VMColor::DisabledNodeBackgroundColor).unwrap(),
            tab_background_color_inactive: config.get_color(VMColor::NodeBackgroundColor).unwrap(),
            tab_active_indicator_color: config.get_color(VMColor::ActiveNodeBorderColor).unwrap(),
        }
    }

    pub fn update_tabs(&mut self, tab_names: &Vec<String>, active_tab: usize) {
        self.tabs.clear();
        for (_index, name) in tab_names.iter().enumerate() {
            self.tabs.push((
                name.clone(),
                None,
                None,
            ));
        }
        self.active_tab = active_tab;
    }
}

impl Widget<()> for VMTabBar {
    fn event(&mut self, ctx: &mut druid::EventCtx, event: &druid::Event, data: &mut (), env: &druid::Env) {
        match event {
            _ => {

            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut druid::LifeCycleCtx, event: &druid::LifeCycle, data: &(), env: &druid::Env) {
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, old_data: &(), data: &(), env: &druid::Env) {
    }

    fn layout(&mut self, ctx: &mut druid::LayoutCtx, bc: &druid::BoxConstraints, data: &(), env: &druid::Env) -> Size {
        for index in 0..self.tabs.len() {
            let mut tab = &mut self.tabs[index];
            if tab.1.is_none() {
                let label = ctx.text().new_text_layout(tab.0.clone())
                .text_color(self.tab_text_color)
                .font(FontFamily::SANS_SERIF, DEFAULT_TAB_LABEL_FONT_SIZE)
                .build()
                .unwrap();
                tab.2 = Some(label.size().ceil());
                tab.1 = Some(label);
            }
        }
        self.tabs.iter().fold(Size::ZERO, |acc, v| {
            return Size {
                height: TAB_BAR_HEIGHT,
                width: acc.width + v.2.unwrap().width + TAB_BAR_LABEL_PADDING_X*2.
        }
        }).ceil()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, _data: &(), _env: &druid::Env) {
        ctx.transform(Affine::translate(Vec2::new(0., -TAB_BAR_HEIGHT)));
        let size = ctx.size();
        let mut x = 0.0;
        for index in 0..self.tabs.len() {
            let cell_width = self.tabs[index].2.unwrap().ceil().width + TAB_BAR_LABEL_PADDING_X*2.; 
            if index == self.active_tab {
                ctx.fill(Rect::new(x, 0.0, x+cell_width, size.height), &self.tab_background_color_active);
                ctx.fill(Rect::new(
                    x, 
                    size.height-TAB_BAR_INDICATOR_HEIGHT, 
                    x + cell_width,
                    size.height,
                ), &self.tab_active_indicator_color);
            } else {
                ctx.fill(Rect::new(x, 0.0, x+cell_width, size.height), &self.tab_background_color_inactive);
            }
            if index != self.tabs.len()-1 {
                let color = Color::BLACK;
                ctx.paint_with_z_index(5, move |ctx| {
                    ctx.fill(Rect::new(x+cell_width, 0.0, x+cell_width+TAB_DIVIDER_WIDTH, size.height), &color);
                });
            }
            self.tabs[index].1.as_mut().unwrap().draw(Point::new(x + TAB_BAR_LABEL_PADDING_X, TAB_BAR_LABEL_PADDING_Y), ctx);
            x += cell_width;
        }
    }
}