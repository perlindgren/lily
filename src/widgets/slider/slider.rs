use crate::util::RangeExt;
use femtovg::{Paint, Path};
use glam::Vec2;
use std::{marker::PhantomData, ops::RangeInclusive};
use vizia::*;

const VERTICAL: bool = true;
const HORIZONTAL: bool = false;

pub struct Slider<L>
where
    L: Lens<Target = f32>,
{
    value: PhantomData<L>,
    range: PhantomData<RangeInclusive<f32>>,
    on_changing: Option<Box<dyn Fn(&mut Context, f32)>>,
}

pub enum InternalEvent {
    Changing(f32),
}

impl<L> Slider<L>
where
    L: Lens<Target = f32>,
{
    pub fn new(cx: &mut Context, value: L, range: RangeInclusive<f32>) -> Handle<Self> {
        Self {
            value: PhantomData::default(),
            on_changing: None,
            range: PhantomData::default(),
        }
        .build2(cx, |cx| {
            // Foreground interactive slider
            SliderBar::new(cx, value.clone(), range.clone())
                .class("bar")
                .on_changing(|cx, value| cx.emit(InternalEvent::Changing(value)));
        })
    }
}

impl<L> View for Slider<L>
where
    L: Lens<Target = f32>,
{
    fn element(&self) -> Option<String> {
        Some("slider".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(ev) = event.message.downcast::<InternalEvent>() {
            match ev {
                InternalEvent::Changing(value) => {
                    if let Some(callback) = &self.on_changing {
                        (callback)(cx, *value);
                    }
                }
            }
        }
    }
}

pub struct SliderBar<L>
where
    L: Lens<Target = f32>,
{
    value: L,
    range: RangeInclusive<f32>,
    hover: bool,
    active: bool,
    last_mouse_pos: Option<Vec2>,
    on_changing: Option<Box<dyn Fn(&mut Context, f32)>>,
}

impl<L> View for SliderBar<L>
where
    L: Lens<Target = f32>,
{
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(ev) = event.message.downcast::<WindowEvent>().cloned() {
            match ev {
                WindowEvent::MouseEnter => self.hover = true,
                WindowEvent::MouseLeave => {
                    self.hover = false;
                }
                WindowEvent::MouseDown(button) => {
                    if button == MouseButton::Left {
                        cx.capture();
                        self.active = true;
                    }
                }
                WindowEvent::MouseUp(button) => {
                    if button == MouseButton::Left {
                        cx.release();
                        self.active = false;
                        self.last_mouse_pos = None;
                    }
                }
                WindowEvent::MouseMove(x, y) => {
                    if self.active {
                        let pos = Vec2::new(x, y);
                        if self.last_mouse_pos.is_none() {
                            self.last_mouse_pos = Some(pos);
                        }

                        // TODO: this isn't really working well...
                        if let Some(callback) = &self.on_changing {
                            // determine whether we are reacting to a vertical or horizontal slider
                            let rect = cx.cache.get_bounds(cx.current);
                            let orientation = rect.h > rect.w;
                            let delta = match orientation {
                                VERTICAL => pos.y - self.last_mouse_pos.unwrap().y,
                                HORIZONTAL => pos.x - self.last_mouse_pos.unwrap().x,
                            };
                            // TODO: Determine scalar based on size
                            let scalar = match cx.modifiers.contains(Modifiers::SHIFT) {
                                true => 0.01,
                                false => 0.05,
                            };
                            let delta_scaled = delta * scalar;
                            // Scale the value to just the small area of our widget
                            let mut new_val = delta_scaled + *self.value.get(cx);

                            // special checks for ranges of negative width
                            if self.range.width().signum() == -1f32 {
                                if new_val > *self.range.start() {
                                    new_val = *self.range.start();
                                } else if new_val < *self.range.end() {
                                    new_val = *self.range.end();
                                }
                            } else {
                                new_val = new_val.clamp(*self.range.start(), *self.range.end());
                            }

                            (callback)(cx, new_val);
                        }
                        self.last_mouse_pos =
                            Some(self.last_mouse_pos.unwrap() + pos - self.last_mouse_pos.unwrap());
                    }
                }
                _ => (),
            }
        }
    }
    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let background_color = cx
            .style
            .background_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default();
        let active_color = cx
            .style
            .border_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default();

        let mut rect = cx.cache.get_bounds(cx.current);

        // determine whether we are drawing a vertical or horizontal slider
        let orientation = rect.h > rect.w;

        match orientation {
            VERTICAL => {
                let old_height = rect.h;
                rect.h = rect.height() * self.range.map(*self.value.get(cx));
                // A little trick since values start from the top and we want
                // the slider to start at the bottom and go up
                rect.y += old_height - rect.h;
            }
            HORIZONTAL => rect.w = rect.width() * self.range.map(*self.value.get(cx)),
        };

        // Draw bar background
        let mut path = Path::new();
        path.rect(rect.x, rect.y, rect.w, rect.h);
        canvas.fill_path(&mut path, Paint::color(background_color.into()));

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

        canvas.fill_path(&mut path, Paint::color(active_color.into()));
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
            last_mouse_pos: None,
        }
        .build(cx)
    }
}

pub trait SliderHandle<P>
where
    P: Lens<Target = f32>,
{
    fn on_changing<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
}

impl<'a, P> SliderHandle<P> for Handle<'a, Slider<P>>
where
    P: Lens<Target = f32>,
{
    fn on_changing<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(slider) = view.downcast_mut::<Slider<P>>() {
                slider.on_changing = Some(Box::new(callback));
            }
        }
        self
    }
}

impl<'a, P> SliderHandle<P> for Handle<'a, SliderBar<P>>
where
    P: Lens<Target = f32>,
{
    fn on_changing<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(slider) = view.downcast_mut::<SliderBar<P>>() {
                slider.on_changing = Some(Box::new(callback));
            }
        }
        self
    }
}
