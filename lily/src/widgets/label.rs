use vizia::{Context, Handle, Lens, View};

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
    fn element(&self) -> Option<String> {
        Some("lily-label".to_string())
    }

    fn event(&mut self, cx: &mut vizia::Context, event: &mut vizia::Event) {}

    fn draw(&self, cx: &mut vizia::DrawContext, canvas: &mut vizia::Canvas) {
        // let entity = cx.current();
        // let rect = cx.cache().get_bounds(entity);
        // TODO: Draw text
    }
}
