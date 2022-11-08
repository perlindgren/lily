use vizia::context::{Context, DrawContext};
use vizia::events::Event;
use vizia::prelude::*;
use vizia::{vg, vg::renderer::OpenGl};

pub struct DragLabel<L>
where
    L: Lens<Target = String>,
{
    data: L,
}

impl<L> DragLabel<L>
where
    L: Lens<Target = String>,
{
    pub fn new(cx: &mut Context, data: L) -> Handle<Self> {
        Self { data }.build(cx, |_| {})
    }
}

impl<L> View for DragLabel<L>
where
    L: Lens<Target = String>,
{
    fn element(&self) -> Option<&'static str> {
        Some("lily-label")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}

    fn draw(&self, cx: &mut DrawContext, canvas: &mut vizia::vg::Canvas<OpenGl>) {
        // let entity = cx.current();
        // let rect = cx.cache().get_bounds(entity);
        // TODO: Draw text
    }
}
