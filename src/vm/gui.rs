use eframe::egui;
use egui::{
    plot::{Line, LineStyle, Plot, Value, Values},
    Color32, InnerResponse, Ui,
};
use polars::prelude::DataFrame;

enum AppType {
    Plot,
    Histogram,
}

pub struct App {
    app_type: AppType,
    data: DataFrame,
    line_style: LineStyle,
}

impl App {
    fn new(data: DataFrame, app_type: AppType) -> Self {
        Self {
            app_type,
            data,
            line_style: LineStyle::dotted_loose(),
        }
    }

    pub fn new_plot(data: DataFrame) -> Self {
        App::new(data, AppType::Plot)
    }

    pub fn new_histogram(data: DataFrame) -> Self {
        App::new(data, AppType::Histogram)
    }

    fn plot_line(&self) -> Line {
        let column_1 = self.data["column_1"].f64().unwrap();
        let column_2 = self.data["column_2"].f64().unwrap();
        let iter = column_1
            .into_iter()
            .zip(column_2.into_iter())
            .map(|(x_chunk, y_chunk)| {
                let x: f64 = x_chunk.unwrap();
                let y: f64 = y_chunk.unwrap();
                Value::new(x, y)
            });
        Line::new(Values::from_values_iter(iter))
            .color(Color32::BLUE)
            .style(self.line_style)
    }

    fn ui(&self, ui: &mut Ui) -> InnerResponse<()> {
        match self.app_type {
            AppType::Plot => Plot::new("raoul").show(ui, |plot_ui| {
                plot_ui.line(self.plot_line());
            }),
            _ => todo!(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.ui(ui));
    }
}
