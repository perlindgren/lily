use std::collections::HashMap;

use glam::Vec2;
use lily_derive::Handle;
use vizia::prelude::*;
use vizia::vg;

use crate::util::BoundingBoxExt;

/// Controls a single point along a normalized XY axis `(-1,-1)..=(1,1)`.
#[derive(Handle)]
pub struct XyPad<P>
where
    P: Lens<Target = Vec2>,
{
    point: P,
    offset: Vec2,
    state: InternalState,
    // Temporary workaround until we can get custom css stuff directly
    classes: HashMap<&'static str, Entity>,
    #[callback(Vec2)]
    on_changing_point: Option<Box<dyn Fn(&mut Context, Vec2)>>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InternalState {
    NoOp,
    Hovering,
    Dragging,
}

impl<P> XyPad<P>
where
    P: Lens<Target = Vec2>,
{
    pub fn new(cx: &mut Context, point: P) -> Handle<Self> {
        let mut classes = HashMap::<&'static str, Entity>::default();
        let mut insert_color = |name| {
            let e = Element::new(cx).class(name).display(Display::None).entity;
            classes.insert(name, e);
        };
        insert_color("point");
        insert_color("crosshair");
        Self {
            point,
            on_changing_point: None,
            state: InternalState::NoOp,
            classes,
            offset: Vec2::ZERO,
        }
        .build(cx, |_| {})
    }
}

impl<P> View for XyPad<P>
where
    P: Lens<Target = Vec2>,
{
    fn element(&self) -> Option<&'static str> {
        Some("xy")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // If clicking and hovered, set the state to dragging
        event.map(|ev: &WindowEvent, _| match *ev {
            WindowEvent::MouseEnter => {
                if self.state != InternalState::Dragging {
                    self.state = InternalState::Hovering;
                }
            }
            WindowEvent::MouseLeave => {
                if self.state != InternalState::Dragging {
                    self.state = InternalState::NoOp;
                }
            }
            WindowEvent::MouseMove(x, y) => {
                if let InternalState::Dragging = self.state {
                    let mouse_pos = Vec2::new(x, y);
                    let mouse_pos_scaled = cx
                        .cache
                        .get_bounds(cx.current())
                        .map_ui_point_unbounded(mouse_pos, true);
                    let final_value = (mouse_pos_scaled + self.offset)
                        .clamp(Vec2::splat(-1f32), Vec2::splat(1f32));
                    if let Some(callback) = &self.on_changing_point {
                        (callback)(cx, final_value);
                    }
                }
            }
            WindowEvent::MouseDown(button) => {
                if button == MouseButton::Left {
                    cx.capture();
                    if self.state == InternalState::Hovering {
                        self.state = InternalState::Dragging;
                        // Set the offset
                        let rect = cx.cache.get_bounds(cx.current);
                        let cursor_pos_scaled = rect.map_ui_point_unbounded(
                            (cx.mouse.cursorx, cx.mouse.cursory).into(),
                            true,
                        );
                        self.offset = self.point.get(cx) - cursor_pos_scaled;
                    }
                }
            }
            WindowEvent::MouseUp(button) => {
                let cursor_pos: Vec2 = (cx.mouse.cursorx, cx.mouse.cursory).into();
                let rect = cx.cache.get_bounds(cx.current);

                if button == MouseButton::Left {
                    cx.release();
                    self.offset = Vec2::ZERO;
                    self.state = if rect.contains_point(cursor_pos) {
                        InternalState::Hovering
                    } else {
                        InternalState::NoOp
                    }
                }
            }
            _ => (),
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let rect = cx.bounds();
        let bg = cx.background_color().copied().unwrap_or_default();
        let border = cx.border_color().copied().unwrap_or_default();

        // Draw background shapes
        // Background
        let mut path = Path::new();
        path.rect(rect.x, rect.y, rect.w, rect.h);
        canvas.fill_path(&mut path, &Paint::color(bg.into()));

        // XY center lines
        let (center_top_x, center_top_y) = rect.center_top();
        let (center_bottom_x, center_bottom_y) = rect.center_bottom();
        let (center_left_x, center_left_y) = rect.center_left();
        let (center_right_x, center_right_y) = rect.center_right();

        let mut path = Path::new();
        path.move_to(center_top_x, center_top_y);
        path.line_to(center_bottom_x, center_bottom_y);
        path.move_to(center_left_x, center_left_y);
        path.line_to(center_right_x, center_right_y);

        // Circle reference lines
        let (center_x, center_y) = rect.center();
        for scale in [1.0, 0.66, 0.33] {
            path.circle(center_x, center_y, (rect.w / 2f32) * scale);
        }
        canvas.stroke_path(&mut path, Paint::color(border.into()));

        // Data point
        self.point.view(cx.data().unwrap(), |point| {
            let point = *point.unwrap();
            let point_entity = *self.classes.get("point").unwrap();
            let ui_point = rect.map_data_point(point, true);
            let point_border = cx.border_color(point_entity).cloned().unwrap_or_default();
            let point_color = cx
                .background_color(point_entity)
                .cloned()
                .unwrap_or_default();

            // Draw crosshairs when dragging
            let crosshair_entity = *self.classes.get("crosshair").unwrap();
            let crosshair_color = cx
                .border_color(crosshair_entity)
                .cloned()
                .unwrap_or_default();
            if self.state == InternalState::Dragging {
                let mut path = Path::new();
                path.move_to(ui_point.x, rect.top());
                path.line_to(ui_point.x, rect.bottom());
                path.move_to(rect.left(), ui_point.y);
                path.line_to(rect.right(), ui_point.y);
                canvas.stroke_path(&mut path, Paint::color(crosshair_color.into()));
            }

            // Point fill
            let mut path = Path::new();
            path.circle(ui_point.x, ui_point.y, 4f32);
            canvas.fill_path(&mut path, Paint::color(point_color.into()));

            // Point outline
            let mut path = Path::new();
            match self.state {
                InternalState::Dragging | InternalState::Hovering => {
                    path.circle(ui_point.x, ui_point.y, 8f32)
                }
                _ => (),
            }

            // Get custom CSS info from a display none element

            canvas.stroke_path(
                &mut path,
                Paint::color(point_border.into()).with_line_width(2f32),
            );
        });
    }
}
