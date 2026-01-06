mod button;
mod read_pdf;

use std::{ops::DerefMut, sync::Arc};

use button::Button;
use gpui::{
    App, Application, Bounds, Context, Image, ImageFormat, ImageSource, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use hayro::RenderSettings;

use crate::read_pdf::Pdf;

struct PdfRenderer {
    pdf: Option<read_pdf::Pdf>,
    image: Option<Arc<Image>>,
    index: usize,
    scale: f32,
}

const USE_SVG: bool = false;

impl PdfRenderer {
    pub fn prev(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        let Some(pdf) = &self.pdf else {
            return;
        };

        if self.index == 1 {
            return;
        }
        self.index -= 1;
        self.image = Some(pdf_to_image(pdf, self.index - 1, self.scale));
        window.refresh();
    }

    pub fn next(&mut self, window: &mut Window, _cx: &mut Context<Self>) {
        let Some(pdf) = &self.pdf else {
            return;
        };

        if self.index >= pdf.nth_pages() {
            return;
        }
        self.index += 1;
        self.image = Some(pdf_to_image(pdf, self.index - 1, self.scale));
        window.refresh();
    }

    pub fn set_file(&mut self, path: std::path::PathBuf, cx: &mut Context<Self>) {
        println!("Selected file: {:?}", path);
        let pdf = read_pdf::Pdf::from_file(path).unwrap();
        self.image = Some(pdf_to_image(&pdf, self.index - 1, self.scale));
        self.pdf = Some(pdf);
        self.index = 1;
        cx.refresh_windows();
    }

    pub fn zoom_plus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pdf) = &self.pdf else {
            return;
        };

        self.scale += 0.1;
        println!("scale: {}", self.scale);
        self.image = Some(pdf_to_image(&pdf, self.index - 1, self.scale));
        window.refresh();
    }

    pub fn zoom_minus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pdf) = &self.pdf else {
            return;
        };

        self.scale -= 0.1;
        println!("scale: {}", self.scale);
        self.image = Some(pdf_to_image(&pdf, self.index - 1, self.scale));
        window.refresh();
    }
}

async fn prompt_for_file() -> Option<std::path::PathBuf> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Select a file to open")
        .add_filter("Pdf Files", &["pdf"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await;

    file.map(|x| x.path().to_path_buf())
}

impl Render for PdfRenderer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let nb_pages = if let Some(pdf) = &self.pdf {
            pdf.nth_pages()
        } else {
            0
        };
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size_full()
            //.justify_center()
            .items_start()
            .shadow_lg()
            //.border_1()
            //.border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .w_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new("open-file-button", "Open File".into()).on_click(_cx.listener(
                            |_this, _event, _win, cx| {
                                let this = cx.weak_entity();
                                let app = cx.deref_mut();
                                app.spawn(async move |cx| {
                                    if let Some(path) = prompt_for_file().await {
                                        this.update(cx, |this, cx| this.set_file(path, cx))
                                            .unwrap();
                                    } else {
                                        println!("Dialog cancelled");
                                    }
                                })
                                .detach();
                            },
                        )),
                    )
                    .child(
                        Button::new("prev", "<".into())
                            .on_click(_cx.listener(|this, _, win, cx| this.prev(win, cx))),
                    )
                    .child(format!("{} / {}", self.index, nb_pages))
                    .child(
                        Button::new("next", ">".into())
                            .on_click(_cx.listener(|this, _, win, cx| this.next(win, cx))),
                    )
                    .child(
                        Button::new("minus", "-".into())
                            .on_click(_cx.listener(|this, _, win, cx| this.zoom_minus(win, cx))),
                    )
                    .child(
                        Button::new("plus", "+".into())
                            .on_click(_cx.listener(|this, _, win, cx| this.zoom_plus(win, cx))),
                    ),
            )
            .when_some(self.image.clone(), |this, image| {
                let source = ImageSource::Image(image);
                let i = gpui::img(source)
                    //.w(gpui::relative(1.))
                    //.max_w(px(900.))
                    //.h(px(1000.))
                    .w_full()
                    .h_full()
                    //.h(gpui::relative(0.8))
                    .object_fit(gpui::ObjectFit::Cover);
                this.child(div().child(i.bg(rgb(0xffffff))))
            })
    }
}

fn pdf_to_image(pdf: &Pdf, index: usize, scale: f32) -> Arc<Image> {
    let page = &pdf.pages()[index];
    let image = if USE_SVG {
        let svg = read_pdf::render_page_svg(pdf.interpreter_settings(), page);
        let image = Image::from_bytes(ImageFormat::Svg, svg);
        image
    } else {
        let render_settings = RenderSettings {
            x_scale: scale,
            y_scale: scale,
            ..RenderSettings::default()
        };
        let png = read_pdf::render_page_png(pdf.interpreter_settings(), &render_settings, page);
        let image = Image::from_bytes(ImageFormat::Png, png);
        image
    };
    Arc::new(image)
}

fn main() -> anyhow::Result<()> {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.), px(1200.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| PdfRenderer {
                    pdf: None,
                    image: None,
                    index: 1,
                    scale: 2.0,
                })
            },
        )
        .unwrap();
    });
    Ok(())
}
