use std::ops::Range;
use iced::{Element, Length};
use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartWidget};
use crate::engine::PacketType;
use crate::ui::UIMessage;

pub struct Spectrogram {
    pub user_data: usize,
    pub current_buf: PacketType,
    pub peak_freq: f32,
    pub freq_data: Vec<(i32, f32)>
}

impl Spectrogram {
    pub fn view(&self) -> Element<UIMessage> {
        ChartWidget::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn new() -> Self {
        Self {
            user_data: 0,
            current_buf: vec![],
            peak_freq: 0.0,
            freq_data: vec![]
        }
    }
}

impl Chart<UIMessage> for Spectrogram {
    type State = u64;

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let x_range: Range<i32> = 0..2000;
        let y_range: Range<f32> = 0.0..2048.0;

        let mut chart = builder
            .set_all_label_area_size(40)
            .build_cartesian_2d(x_range, y_range)
            .expect("Failed to build chart");

        // TODO try to avoid cloning here
        let series = LineSeries::new(self.freq_data.clone(), &BLACK);

        chart.configure_mesh()
            .draw()
            .expect("Failed to draw mesh");

        chart.draw_series(series)
            .expect("Failed to draw series");

    }
}

