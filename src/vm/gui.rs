use eframe::egui;
use egui::{
    plot::{Bar, BarChart, Line, LineStyle, Plot, Value, Values},
    Color32, InnerResponse, Ui,
};
use polars::prelude::{ChunkSort, DataFrame};

enum AppType {
    Plot,
    Histogram,
}

pub struct App {
    app_type: AppType,
    bins: Option<usize>,
    data: DataFrame,
    line_style: LineStyle,
}

impl App {
    fn new(data: DataFrame, app_type: AppType, bins: Option<usize>) -> Self {
        Self {
            app_type,
            data,
            line_style: LineStyle::dotted_loose(),
            bins,
        }
    }

    pub fn new_plot(data: DataFrame) -> Self {
        App::new(data, AppType::Plot, None)
    }

    pub fn new_histogram(data: DataFrame, bins: usize) -> Self {
        App::new(data, AppType::Histogram, Some(bins))
    }

    fn plot_line(&self) -> Line {
        let column_1 = self.data["column_1"].f64().unwrap();
        let column_2 = self.data["column_2"].f64().unwrap();
        let iter = column_1
            .into_iter()
            .zip(column_2.into_iter())
            .map(|(x, y)| {
                let x: f64 = x.unwrap();
                let y: f64 = y.unwrap();
                Value::new(x, y)
            });
        println!("{}", iter.len());
        Line::new(Values::from_values_iter(iter))
            .color(Color32::BLUE)
            .style(self.line_style)
    }

    fn plot_histogram(&self) -> BarChart {
        let column = &self.data["column"];
        let bins = self.bins.unwrap();
        let bins_len = column.len() / bins;
        let column = column.f64().unwrap();
        let column: Vec<_> = column.sort(false).into_iter().map(Option::unwrap).collect();
        let column: Vec<_> = column.chunks(bins_len).collect();
        let bars: Vec<_> = column
            .windows(2)
            .map(|v| {
                let first_arr = v[0];
                let second_arr = v[1];
                let value: f64 = first_arr.len().to_string().parse().unwrap();
                let first = first_arr[0];
                let limit = second_arr[0];
                Bar::new(first, value).width((limit - first) * 0.25)
            })
            .collect();
        BarChart::new(bars)
    }

    fn ui(&self, ui: &mut Ui) -> InnerResponse<()> {
        Plot::new("raoul").show(ui, |plot_ui| match self.app_type {
            AppType::Plot => plot_ui.line(self.plot_line()),
            AppType::Histogram => plot_ui.bar_chart(self.plot_histogram()),
        })
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| self.ui(ui));
    }
}
