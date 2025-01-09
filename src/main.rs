use nannou::prelude::*;
use nannou_audio::Buffer;
use nannou_audio::{self as audio};
use ringbuf::{traits::*, HeapCons, HeapProd, HeapRb};

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    last_dot_position_change: f32,
    dot_position_change_timeout: f32,
    dot_positions: Vec<Vec2>,
    in_stream: audio::Stream<InputModel>,
    out_stream: audio::Stream<OutputModel>,
}

struct InputModel {
    pub producer: HeapProd<f32>,
}

struct OutputModel {
    pub consumer: HeapCons<f32>,
}

fn model(app: &App) -> Model {
    app.new_window()
        .key_pressed(key_pressed)
        .view(view)
        .build()
        .unwrap();

    // Initialise the audio host so we can spawn an audio stream.
    let audio_host = audio::Host::new();

    // Create a ring buffer and split it into producer and consumer
    let latency_samples = 1024 * 4;
    let ring_buffer = HeapRb::<f32>::new(latency_samples * 2); // Add some latency
    let (mut prod, cons) = ring_buffer.split();
    for _ in 0..latency_samples {
        // The ring buffer has twice as much space as necessary to add latency here,
        // so this should never fail
        prod.try_push(0.0).unwrap();
    }

    // Create input model and input stream using that model
    let in_model = InputModel { producer: prod };
    let in_stream = audio_host
        .new_input_stream(in_model)
        .capture(pass_in)
        .build()
        .unwrap();

    // Create output model and output stream using that model
    let out_model = OutputModel { consumer: cons };
    let out_stream = audio_host
        .new_output_stream(out_model)
        .render(pass_out)
        .build()
        .unwrap();

    in_stream.play().unwrap();
    out_stream.play().unwrap();

    Model {
        last_dot_position_change: app.time,
        dot_position_change_timeout: 2.0,
        dot_positions: create_random_points(app.window_rect(), 4),
        in_stream,
        out_stream,
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    if model.last_dot_position_change + (model.dot_position_change_timeout as f32) < app.time {
        model.last_dot_position_change = app.time;
        model.dot_position_change_timeout = random_range(1.0, 4.0);
        model.dot_positions = create_random_points(app.window_rect(), random_range(1, 13));
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();

    // Clear the frame
    draw.background().color(STEELBLUE);

    // Get the window dimensions
    let win = app.window_rect();

    // Create a gradient with colors in linear space
    let gradient = nannou::color::Gradient::new(vec![
        LinSrgb::new(
            PLUM.red as f32 / 255.0,
            PLUM.green as f32 / 255.0,
            PLUM.blue as f32 / 255.0,
        ),
        LinSrgb::new(
            STEELBLUE.red as f32 / 255.0,
            STEELBLUE.green as f32 / 255.0,
            STEELBLUE.blue as f32 / 255.0,
        ),
    ]);
    for i in 0..100 {
        let t = i as f32 / 100.0;
        let color = gradient.get(t);
        draw.rect()
            .xy(win.xy())
            .w_h(win.w(), win.h() / 100.0)
            .y(win.bottom() + t * win.h())
            .color(color);
    }

    for pos in model.dot_positions.clone() {
        draw.ellipse()
            .color(rgba(BISQUE.red, BISQUE.green, SIENNA.blue, 50))
            .xy(pos)
            .w_h(
                100.0 + random_range(0.0, 4.0),
                100.0 + random_range(0.0, 4.0),
            );
    }

    let amplitude = 50.0; // Height of the sine wave
    let frequency = 0.05; // Number of cycles per unit length
    let phase_shift = 50.0; // Shift on the x-axis

    // Draw the sine wave
    let points = (0..1000).map(|i| {
        let x = map_range(i, 0, 1000, win.left(), win.right());
        let y =
            win.bottom() + (amplitude * 0.3) + (amplitude * (frequency * (x - phase_shift)).sin());
        pt2(x, y)
    });

    draw.polyline()
        .weight(12.0)
        .color(rgba(SLATEBLUE.red, SLATEBLUE.green, SIENNA.blue, 50))
        .points(points);

    let linepoints = 50;
    let lineweight = 6.0;

    let sine = app.time.sin();
    let gradsine = map_range(sine, -1.0, 1.0, 0.0, 1.0);
    let points = (0..linepoints).map(|i| {
        let x = i as f32 - 25.0; //subtract 25 to center the sine wave
        let point = pt2(x, x.sin() + 4.0) * 20.0; //scale sine wave by 20.0
        (point, gradient.get(i as f32 / linepoints as f32 * gradsine))
    });
    draw.polyline().weight(lineweight).points_colored(points);

    let tangus = app.time.sin();
    let gradtan = map_range(tangus, -1.0, 1.0, 0.0, 1.0);
    let points = (0..linepoints).map(|i| {
        let x = i as f32 - 25.0; //subtract 25 to center the sine wave
        let point = pt2(x, x.tan()) * 20.0; //scale sine wave by 20.0
        (point, gradient.get(i as f32 / linepoints as f32 * gradtan))
    });
    draw.polyline().weight(lineweight).points_colored(points);

    let cosine: f32 = app.time.cos();
    let gradcosine = map_range(cosine, -1.0, 1.0, 0.0, 1.0);
    let points = (0..linepoints).map(|i| {
        let x = i as f32 - 25.0; //subtract 25 to center the sine wave
        let point = pt2(x, x.cos() - 4.0) * 20.0; //scale sine wave by 20.0
        (
            point,
            gradient.get(i as f32 / linepoints as f32 * gradcosine),
        )
    });
    draw.polyline().weight(lineweight).points_colored(points);

    draw.text("Soloheisbeer")
        .font_size(96)
        .w(win.w())
        .xy(win.pad(200.0).mid_top())
        .rotate(gradsine * gradtan * 360.0)
        .color(gradient.get(map_range(cosine, -1.0, 1.0, 0.0, 1.0)));

    draw.to_frame(app, &frame).unwrap();
}

fn create_random_points(win: Rect, point_amount: i32) -> Vec<Vec2> {
    let points = (0..point_amount)
        .map(|_| {
            return pt2(
                map_range(random(), 0.0, 1.0, win.bottom(), win.top()),
                map_range(random(), 0.0, 1.0, win.bottom(), win.top()),
            );
        })
        .collect::<Vec<_>>();
    return points;
}

fn pass_in(model: &mut InputModel, buffer: &Buffer) {
    for frame in buffer.frames() {
        for sample in frame {
            model.producer.try_push(*sample).ok();
        }
    }
}

fn pass_out(model: &mut OutputModel, buffer: &mut Buffer) {
    for frame in buffer.frames_mut() {
        for sample in frame {
            let recorded_sample = match model.consumer.try_pop() {
                Some(f) => f,
                None => 0.0,
            };
            *sample = recorded_sample;
        }
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Space => {
            if model.in_stream.is_paused() {
                model.in_stream.play().unwrap();
                model.out_stream.play().unwrap();
            } else {
                model.in_stream.pause().unwrap();
                model.out_stream.pause().unwrap();
            }
        }
        _ => {}
    }
}
