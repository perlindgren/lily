use baseview::WindowScalePolicy;
use lily::{
    math::Vec2,
    util::{CurvePoint, CurvePoints},
    widgets::*,
    DEFAULT_STYLE,
};
use std::ops::RangeInclusive;
use vizia::*;

#[derive(Lens)]
pub struct AppData {
    xy_data: Vec2,
    mseg_data: CurvePoints,
    mseg_zoom_data: RangeInclusive<f32>,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            xy_data: Vec2::ZERO,
            mseg_zoom_data: 0.0f32..=1.0f32,
            mseg_data: CurvePoints(
                vec![
                    (0f32, 0f32),
                    (0.5f32, 1.0f32),
                    (1.0f32, 0.7f32),
                    (2.0f32, 0.5f32),
                    (3.0f32, 0.0f32),
                ]
                .iter()
                .cloned()
                .map(CurvePoint::from)
                .collect(),
            ),
        }
    }
}

#[derive(Clone, Copy)]
pub enum AppEvent {
    XyControl { point: Vec2 },
    MsegZoomStart { value: f32 },
    MsegZoomEnd { value: f32 },
    MsegPoint { index: usize, pos: Vec2 },
    MsegInsertPoint { index: usize, pos: Vec2 },
    MsegRemovePoint { index: usize },
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        if let Some(event) = event.message.downcast().cloned() {
            match event {
                AppEvent::XyControl { point } => {
                    self.xy_data = point;
                }
                AppEvent::MsegZoomStart { value } => {
                    self.mseg_zoom_data = value..=*self.mseg_zoom_data.end()
                }
                AppEvent::MsegZoomEnd { value } => {
                    self.mseg_zoom_data = *self.mseg_zoom_data.start()..=value
                }
                AppEvent::MsegPoint { index, pos } => {
                    if let Some(p) = self.mseg_data.get_mut(index) {
                        p.x = pos.x;
                        p.y = pos.y
                    }
                }
                AppEvent::MsegInsertPoint { index, pos } => {
                    self.mseg_data.insert(index, CurvePoint::from(pos));
                }
                AppEvent::MsegRemovePoint { index } => {
                    self.mseg_data.remove(index);
                }
            }
        }
    }
}

fn main() {
    let window = WindowDescription::new()
        .with_title("Showcase")
        .with_inner_size(500, 500);
    Application::new(window, |cx| {
        cx.add_theme(DEFAULT_STYLE);
        AppData::default().build(cx);

        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                // XY Pad
                VStack::new(cx, |cx| {
                    XyPad::new(cx, AppData::xy_data)
                        .on_changing_point(|cx, point| cx.emit(AppEvent::XyControl { point }));
                    lily::widgets::Slider::new(cx, AppData::xy_data.map(|pos| pos.x), -1f32..=1f32)
                        .on_changing(|cx, val| {
                            cx.emit(AppEvent::XyControl {
                                point: Vec2::new(val, AppData::xy_data.get(cx).y),
                            });
                        })
                        .height(Pixels(24f32))
                        .width(Pixels(200f32));
                })
                .width(Pixels(200f32));
                lily::widgets::Slider::new(cx, AppData::xy_data.map(|pos| pos.y), 1f32..=-1f32)
                    .on_changing(|cx, val| {
                        cx.emit(AppEvent::XyControl {
                            point: Vec2::new(AppData::xy_data.get(cx).x, val),
                        });
                    })
                    .height(Pixels(200f32))
                    .width(Pixels(24f32))
                    .top(Percentage(0f32));
            });
            // Multi stage envelope generator
            Mseg::new(cx, AppData::mseg_data, AppData::mseg_zoom_data, 8f32)
                .on_changing_range_start(|cx, x| cx.emit(AppEvent::MsegZoomStart { value: x }))
                .on_changing_range_end(|cx, x| cx.emit(AppEvent::MsegZoomEnd { value: x }))
                .on_changing_point(|cx, index, pos| {
                    cx.emit(AppEvent::MsegPoint { index, pos });
                })
                .on_insert_point(|cx, index, pos| cx.emit(AppEvent::MsegInsertPoint { index, pos }))
                .on_remove_point(|cx, index| cx.emit(AppEvent::MsegRemovePoint { index }));
        })
        .background_color(Color::rgb(21, 20, 21))
        .width(Stretch(1f32))
        .height(Stretch(1f32))
        .child_space(Pixels(24f32))
        .row_between(Pixels(24f32));
    })
    // .with_scale_policy(WindowScalePolicy::ScaleFactor(1.25))
    .run();
}
