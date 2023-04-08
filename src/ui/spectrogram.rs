use std::ops::Range;
use std::time::{Duration, Instant};
use iced::{Element, Length};
use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartWidget};
use ringbuf::HeapRb;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::{divide_by_N, divide_by_N_sqrt};
use spectrum_analyzer::windows::hann_window;
use crate::engine::PacketType;
use crate::ui::UIMessage;

pub struct Spectrogram {
    pub user_data: usize,
    pub current_buf: PacketType
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
            current_buf: vec![]
        }
    }
}

impl Chart<UIMessage> for Spectrogram {
    type State = u64;

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let x_range: Range<i32> = 0..8000;
        let y_range: Range<f32> = 0.0..2048.0;

        let mut chart = builder
            .set_all_label_area_size(40)
            .build_cartesian_2d(x_range, y_range)
            .expect("Failed to build chart");

        let points = if !self.current_buf.is_empty() {
            let hann_window = hann_window(self.current_buf.as_slice());
            let spectrum = samples_fft_to_spectrum(
                &hann_window,
                48000,
                FrequencyLimit::Max(12000.0),
                Some(&divide_by_N_sqrt))
                .expect("Could not extract frequncy spectrum");

            let points: Vec<(i32, f32)> = spectrum.data()
                .iter()
                .map(|(freq, amp)| {
                    (freq.val() as i32, amp.val() * 1000.0)
                }).collect();

            points
        } else {
            vec![]
        };

        let series = LineSeries::new(points, &BLACK);

        chart.configure_mesh()
            .draw()
            .expect("Failed to draw mesh");

        chart.draw_series(series)
            .expect("Failed to draw series");

    }
}

