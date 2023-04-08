use iced::{Element, Length};
use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters_iced::{Chart, ChartWidget};
use ringbuf::HeapRb;
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum};
use spectrum_analyzer::scaling::divide_by_N;
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
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .set_all_label_area_size(40)
            .build_cartesian_2d(0..18000, 0.0..2048.0 as f32)
            .expect("Failed to build chart");

        /*
        let points: Vec<(i32, i32)> = self.current_buf.iter()
            .enumerate()
            .map(|(a, b)| (a as i32, *b as i32))
            .collect();
         */


        let points = if !self.current_buf.is_empty() {
            let hann_window = hann_window(self.current_buf.as_slice());
            let spectrum = samples_fft_to_spectrum(
                &hann_window,
                48000,
                FrequencyLimit::Max(12000.0),
                Some(&divide_by_N))
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

        /*
        let points: Vec<(i32, f32)> = self.current_buf.iter()
            .enumerate()
            .map(|(a, b)| {
                (a as i32, 1024.0 * *b)
            })
            .collect();
         */

        let series = LineSeries::new(points, &BLACK);

        chart.configure_mesh()
            .draw()
            .expect("Failed to draw mesh");

        chart.draw_series(series)
            .expect("Failed to draw series");
    }
}

