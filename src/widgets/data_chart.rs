// Copyright (c) 2025 Oscar Pernia
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

/// A data point with timestamp and value
#[derive(Clone, Debug)]
pub struct DataPoint {
    pub timestamp: f64,
    pub value: f64,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct MQTTyDataChart {
        /// Store data points (timestamp, value)
        pub data_points: RefCell<VecDeque<DataPoint>>,
        /// Maximum number of points to display
        pub max_points: RefCell<usize>,
        /// Topic being charted
        pub topic: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MQTTyDataChart {
        const NAME: &'static str = "MQTTyDataChart";
        type Type = super::MQTTyDataChart;
        type ParentType = gtk::DrawingArea;
    }

    impl ObjectImpl for MQTTyDataChart {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            *self.max_points.borrow_mut() = 100;

            obj.set_content_width(400);
            obj.set_content_height(200);

            // Set up the draw function
            obj.set_draw_func(glib::clone!(
                #[weak]
                obj,
                move |_, cr, width, height| {
                    obj.draw(cr, width, height);
                }
            ));
        }
    }

    impl WidgetImpl for MQTTyDataChart {}
    impl DrawingAreaImpl for MQTTyDataChart {}
}

glib::wrapper! {
    pub struct MQTTyDataChart(ObjectSubclass<imp::MQTTyDataChart>)
        @extends gtk::Widget, gtk::DrawingArea,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl MQTTyDataChart {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_topic(&self, topic: &str) {
        *self.imp().topic.borrow_mut() = topic.to_string();
    }

    pub fn topic(&self) -> String {
        self.imp().topic.borrow().clone()
    }

    /// Add a new data point
    pub fn add_point(&self, value: f64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let mut data = self.imp().data_points.borrow_mut();
        let max = *self.imp().max_points.borrow();

        data.push_back(DataPoint { timestamp, value });

        // Remove oldest points if we exceed max
        while data.len() > max {
            data.pop_front();
        }

        drop(data);
        self.queue_draw();
    }

    /// Clear all data points
    pub fn clear(&self) {
        self.imp().data_points.borrow_mut().clear();
        self.queue_draw();
    }

    /// Set maximum number of points to display
    pub fn set_max_points(&self, max: usize) {
        *self.imp().max_points.borrow_mut() = max;
    }

    /// Draw the chart
    fn draw(&self, cr: &gtk::cairo::Context, width: i32, height: i32) {
        let width = width as f64;
        let height = height as f64;
        let padding = 40.0;

        // Get style context for colors
        let style = self.style_context();
        let fg_color = style.color();

        // Background
        cr.set_source_rgba(0.1, 0.1, 0.1, 1.0);
        cr.rectangle(0.0, 0.0, width, height);
        let _ = cr.fill();

        let data = self.imp().data_points.borrow();

        if data.is_empty() {
            // Draw "No data" message
            cr.set_source_rgba(
                fg_color.red() as f64,
                fg_color.green() as f64,
                fg_color.blue() as f64,
                0.5,
            );
            cr.select_font_face("Sans", gtk::cairo::FontSlant::Normal, gtk::cairo::FontWeight::Normal);
            cr.set_font_size(14.0);
            let text = "No numeric data";
            let extents = cr.text_extents(text).unwrap();
            cr.move_to(
                (width - extents.width()) / 2.0,
                (height + extents.height()) / 2.0,
            );
            let _ = cr.show_text(text);
            return;
        }

        // Calculate min/max values
        let (min_val, max_val) = data.iter().fold((f64::MAX, f64::MIN), |(min, max), p| {
            (min.min(p.value), max.max(p.value))
        });

        let min_time = data.front().map(|p| p.timestamp).unwrap_or(0.0);
        let max_time = data.back().map(|p| p.timestamp).unwrap_or(1.0);

        // Add some padding to value range
        let value_range = if (max_val - min_val).abs() < 0.001 {
            1.0
        } else {
            max_val - min_val
        };
        let time_range = if (max_time - min_time).abs() < 0.001 {
            1.0
        } else {
            max_time - min_time
        };

        let chart_width = width - padding * 2.0;
        let chart_height = height - padding * 2.0;

        // Draw grid lines
        cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
        cr.set_line_width(0.5);

        // Horizontal grid lines (5 lines)
        for i in 0..=5 {
            let y = padding + (chart_height * i as f64 / 5.0);
            cr.move_to(padding, y);
            cr.line_to(width - padding, y);
            let _ = cr.stroke();

            // Y-axis labels
            let value = max_val - (value_range * i as f64 / 5.0);
            cr.set_source_rgba(
                fg_color.red() as f64,
                fg_color.green() as f64,
                fg_color.blue() as f64,
                0.7,
            );
            cr.set_font_size(10.0);
            let label = format!("{:.1}", value);
            cr.move_to(5.0, y + 4.0);
            let _ = cr.show_text(&label);
            cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
        }

        // Draw axes
        cr.set_source_rgba(0.5, 0.5, 0.5, 1.0);
        cr.set_line_width(1.0);

        // Y-axis
        cr.move_to(padding, padding);
        cr.line_to(padding, height - padding);
        let _ = cr.stroke();

        // X-axis
        cr.move_to(padding, height - padding);
        cr.line_to(width - padding, height - padding);
        let _ = cr.stroke();

        // Draw the data line
        cr.set_source_rgba(0.3, 0.7, 1.0, 1.0);
        cr.set_line_width(2.0);

        let mut first = true;
        for point in data.iter() {
            let x = padding + ((point.timestamp - min_time) / time_range) * chart_width;
            let y = padding + ((max_val - point.value) / value_range) * chart_height;

            if first {
                cr.move_to(x, y);
                first = false;
            } else {
                cr.line_to(x, y);
            }
        }
        let _ = cr.stroke();

        // Draw data points
        cr.set_source_rgba(0.4, 0.8, 1.0, 1.0);
        for point in data.iter() {
            let x = padding + ((point.timestamp - min_time) / time_range) * chart_width;
            let y = padding + ((max_val - point.value) / value_range) * chart_height;

            cr.arc(x, y, 3.0, 0.0, 2.0 * std::f64::consts::PI);
            let _ = cr.fill();
        }

        // Draw topic name
        cr.set_source_rgba(
            fg_color.red() as f64,
            fg_color.green() as f64,
            fg_color.blue() as f64,
            0.8,
        );
        cr.set_font_size(12.0);
        let topic = self.imp().topic.borrow();
        if !topic.is_empty() {
            cr.move_to(padding + 5.0, padding - 5.0);
            let _ = cr.show_text(&topic);
        }

        // Draw current value
        if let Some(last) = data.back() {
            let value_text = format!("Current: {:.2}", last.value);
            let extents = cr.text_extents(&value_text).unwrap();
            cr.move_to(width - padding - extents.width() - 5.0, padding - 5.0);
            let _ = cr.show_text(&value_text);
        }
    }

    /// Try to parse a payload as a numeric value
    pub fn try_add_from_payload(&self, payload: &str) -> bool {
        // Try to parse as plain number
        if let Ok(value) = payload.trim().parse::<f64>() {
            self.add_point(value);
            return true;
        }

        // Try to parse as JSON and extract a numeric value
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
            if let Some(value) = Self::extract_numeric_value(&json) {
                self.add_point(value);
                return true;
            }
        }

        false
    }

    /// Extract a numeric value from JSON
    fn extract_numeric_value(json: &serde_json::Value) -> Option<f64> {
        match json {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::Object(obj) => {
                // Try common field names for numeric values
                for key in ["value", "val", "data", "temp", "temperature", "humidity", "pressure", "reading"] {
                    if let Some(v) = obj.get(key) {
                        if let Some(n) = Self::extract_numeric_value(v) {
                            return Some(n);
                        }
                    }
                }
                // Try the first numeric value found
                for (_, v) in obj.iter() {
                    if let Some(n) = Self::extract_numeric_value(v) {
                        return Some(n);
                    }
                }
                None
            }
            serde_json::Value::Array(arr) => {
                // Try the first element
                arr.first().and_then(Self::extract_numeric_value)
            }
            _ => None,
        }
    }
}

impl Default for MQTTyDataChart {
    fn default() -> Self {
        Self::new()
    }
}
