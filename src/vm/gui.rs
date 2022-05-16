use eframe::egui;
use egui::{
    plot::{Bar, BarChart, Line, LineStyle, Plot, Value, Values},
    Color32, InnerResponse, Ui,
};
use polars::prelude::DataFrame;

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
        Line::new(Values::from_values_iter(iter))
            .color(Color32::BLUE)
            .style(self.line_style)
    }

    fn plot_histogram(&self) -> BarChart {
        let bins = self.bins.unwrap() + 1;
        let mut data: Vec<(f64, f64)> = vec![(0.0, f64::MAX); bins];
        let column = &self.data["column"];
        let min = column.min::<f64>().unwrap();
        let max = column.max::<f64>().unwrap();
        let step = (max - min) / bins.to_string().parse::<f64>().unwrap();
        let chunked_arr = column.f64().unwrap();
        chunked_arr.into_iter().for_each(|v| {
            let value = v.unwrap();
            let index: usize = match (value - min) / step {
                x if x >= (bins as f64) => bins - 1,
                x => x.floor().to_string().parse().unwrap(),
            };
            let (count, start) = data.get_mut(index).unwrap();
            *count += 1.0;
            if *start > value {
                *start = value;
            }
        });
        let bars: Vec<Bar> = data
            .windows(2)
            .map(|v| {
                let (count, start) = v[0];
                let limit = v[1].1;
                Bar::new(start, count).width((limit - start) * 0.95)
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
