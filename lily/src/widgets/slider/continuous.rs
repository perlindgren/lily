use crate::util::{BoundingBoxExt, RangeExt};
use glam::Vec2;
use lily_derive::Handle;
use std::{marker::PhantomData, ops::RangeInclusive};
// use vizia::context::Context;
use vizia::prelude::*;
use vizia::vg::{Paint, Path};

const VERTICAL: bool = true;
const HORIZONTAL: bool = false;

#[derive(Handle)]
pub struct DragSlider<L>
where
    L: Lens<Target = f32>,
{
    value: PhantomData<L>,
    range: PhantomData<RangeInclusive<f32>>,
    #[callback(f32)]
    on_changing: Option<Box<dyn Fn(&mut EventContext, f32)>>,
}

pub enum InternalEvent {
    Changing(f32),
}

impl<L> DragSlider<L>
where
    L: Lens<Target = f32>,
{
    /// Create a new `DragSlider`. Depending on if dimensions are portrait or
    /// landscape, it will automatically choose horizontal or vertical mode
    ///
    /// # Parameters
    ///
    /// * `cx` - Vizia `Context`
    /// * `value` - a `vizia::Lens` specifying the value of this slider
    /// * `range` - the arbitrary range of this slider. In most cases, you'll
    ///   want `0f32..=1f32` or `-1f32..=1f32` for a centered slider.
    pub fn new(cx: &mut Context, value: L, range: RangeInclusive<f32>) -> Handle<Self> {
        Self {
            value: PhantomData::default(),
            on_changing: None,
            range: PhantomData::default(),
        }
        .build(cx, |cx| {
            // Foreground interactive slider
            SliderBar::new(cx, value.clone(), range.clone())
                .class("bar")
                .on_changing(|cx, value| cx.emit(InternalEvent::Changing(value)));
        })
    }
}

impl<L> View for DragSlider<L>
where
    L: Lens<Target = f32>,
{
    fn element(&self) -> Option<&'static str> {
        Some("slider")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|ev: &InternalEvent, _| match *ev {
            InternalEvent::Changing(value) => {
                if let Some(callback) = &self.on_changing {
                    (callback)(cx, value);
                }
            }
        });
    }
}
#[derive(Handle)]
pub struct SliderBar<L>
where
    L: Lens<Target = f32>,
{
    value: L,
    range: RangeInclusive<f32>,
    hover: bool,
    active: bool,
    /// The offset of the cursor to the handle, set when clicking. This ensures
    /// that values don't skip when first dragging to to cursor position
    offset: f32,
    #[callback(f32)]
    on_changing: Option<Box<dyn Fn(&mut EventContext, f32)>>,
}

impl<L> View for SliderBar<L>
where
    L: Lens<Target = f32>,
{
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|ev: &WindowEvent, _| match *ev {
            WindowEvent::MouseEnter => self.hover = true,
            WindowEvent::MouseLeave => {
                self.hover = false;
            }
            WindowEvent::MouseDown(button) => {
                if button == MouseButton::Left {
                    cx.capture();
                    self.active = true;
                    // set the offset
                    let rect = cx.cache.get_bounds(cx.current());
                    let mouse_pos: Vec2 = (cx.mouse.cursorx, cx.mouse.cursory).into();
                    // get the difference between the mapped mouse pos and
                    // the current value
                    let mouse_mapped = rect.map_ui_point(mouse_pos, true);
                    self.offset = self.value.get(cx)
                        - match rect.h > rect.w {
                            VERTICAL => mouse_mapped.y,
                            HORIZONTAL => mouse_mapped.x,
                        };
                }
            }
            WindowEvent::MouseUp(button) => {
                if button == MouseButton::Left {
                    cx.release();
                    self.active = false;
                }
                // reset offset
                self.offset = 0f32;
            }
            // TODO: figure out a way to not rely on the map_ui_point and
            // instead just have some sort of scalar
            WindowEvent::MouseMove(x, y) => {
                if self.active {
                    if let Some(callback) = &self.on_changing {
                        // determine whether we are reacting to a vertical
                        // or horizontal slider
                        let rect = cx.cache.get_bounds(cx.current());
                        let orientation = rect.h > rect.w;
                        // let scalar = match
                        //     cx.modifiers.contains(Modifiers::SHIFT) {
                        //     true => 0.1, false => 1f32, };
                        let mut val = {
                            // let mapped = rect.map_ui_point((x, y).into(),
                            // true);
                            let ratio = rect.map_ui_point_unbounded((x, y).into(), true);
                            self.offset
                                + match orientation {
                                    VERTICAL => ratio.y,
                                    HORIZONTAL => ratio.x,
                                }
                        };

                        // TODO: Determine scalar based on size

                        // let delta_scaled = delta * scalar; Scale the
                        // value to just the small area of our widget let
                        // mut new_val = delta_scaled + self.value.get(cx);

                        // special checks for ranges of negative width
                        if self.range.width().signum() == -1f32 {
                            if val > *self.range.start() {
                                val = *self.range.start();
                            } else if val < *self.range.end() {
                                val = *self.range.end();
                            }
                        } else {
                            val = val.clamp(*self.range.start(), *self.range.end());
                        }

                        (callback)(cx, val);
                    }
                }
            }
            _ => (),
        });
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let background_color = cx.background_color().cloned().unwrap_or_default();
        let active_color = cx.border_color().cloned().unwrap_or_default();

        let mut rect = cx.bounds();

        // determine whether we are drawing a vertical or horizontal slider
        let orientation = rect.h > rect.w;

        self.value.view(cx.data().unwrap(), |value| {
            match orientation {
                VERTICAL => {
                    let old_height = rect.h;
                    rect.h = rect.height() * self.range.map(value.cloned().unwrap_or_default());
                    // A little trick since values start from the top and we
                    // want the slider to start at the bottom and go up
                    rect.y += old_height - rect.h;
                }
                HORIZONTAL => {
                    rect.w = rect.width() * self.range.map(value.cloned().unwrap_or_default())
                }
            }
        });

        // Draw bar background
        let mut path = Path::new();
        path.rect(rect.x, rect.y, rect.w, rect.h);
        canvas.fill_path(&mut path, &Paint::color(background_color.into()));

        // Draw bar line control
        let mut path = Path::new();
        let bar_thickness = if self.active {
            6f32
        } else if self.hover {
            4f32
        } else {
            2f32
        };

        match orientation {
            VERTICAL => path.rect(
                rect.left(),
                rect.top() - (bar_thickness / 2f32),
                rect.width(),
                bar_thickness,
            ),
            HORIZONTAL => path.rect(
                rect.right() - (bar_thickness / 2f32),
                rect.top(),
                bar_thickness,
                rect.height(),
            ),
        };

        canvas.fill_path(&mut path, &Paint::color(active_color.into()));
    }
}
impl<L> SliderBar<L>
where
    L: Lens<Target = f32>,
{
    fn new(cx: &mut Context, value: L, range: RangeInclusive<f32>) -> Handle<Self> {
        Self {
            value,
            on_changing: None,
            range,
            hover: false,
            active: false,
            offset: 0f32,
        }
        .build(cx, |_| {})
    }
}

// impl<'a, P> SliderBarHandle<P> for Handle<'a, SliderBar<P>> where P:
// Lens<Target = f32>, { fn on_changing<F>(self, callback: F) -> Self where F:
//     'static + Fn(&mut Context, f32), { if let Some(view) =
// self.cx.views.get_mut(&self.entity) { if let Some(slider) =
//     view.downcast_mut::<SliderBar<P>>() { slider.on_changing =
//     Some(Box::new(callback)); } } self } }
