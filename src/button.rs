use gpui::{App, ClickEvent, ElementId, SharedString, Window, div, prelude::*, rgb};

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: SharedString) -> Self {
        Self {
            id: id.into(),
            label,
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .flex()
            .text_sm()
            .border_2()
            .p_2()
            .rounded_lg()
            .border_color(rgb(0xddcccc))
            .text_color(rgb(0xffffff))
            .bg(rgb(0x000000))
            .hover(|style| style.bg(rgb(0x333333)))
            .when_some(self.on_click, |this, on_click| {
                this.on_click(move |evt, win, cx| (on_click)(evt, win, cx))
            })
            .child(self.label)
    }
}
