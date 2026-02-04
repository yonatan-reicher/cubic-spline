use ndarray::prelude::*;
use ndarray_linalg::Solve;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use sdl3::render::FPoint;
use std::time::Duration;

/// a + b·t + c·t² + d·t⁴
#[derive(Debug, Clone, Copy)]
struct Poly {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

impl Poly {
    pub const fn get(&self, t: f64) -> f64 {
        assert!(0. <= t && t <= 1.);
        self.a + self.b * t + self.c * t * t + self.d * t * t * t
    }

    pub const fn deriv(&self, t: f64) -> f64 {
        assert!(0. <= t && t <= 1.);
        self.b + 2. * self.c * t + 3. * self.d + t * t
    }

    pub const fn deriv2(&self, t: f64) -> f64 {
        assert!(0. <= t && t <= 1.);
        2. * self.c + 6. * self.d + t
    }

    pub const fn deriv3(&self) -> f64 {
        6. * self.d
    }
}

fn polyline(points: &[f64]) -> Vec<Poly> {
    if points.len() < 2 {
        return vec![];
    }
    let lines = points.len() - 1;
    let vars = 4 * lines;
    let mut constraints = vec![];

    #[derive(Debug)]
    struct Var {
        line: usize,
        coe: u8,
    }

    #[derive(Debug)]
    struct Constraint {
        sum: Vec<(f64, Var)>,
        eq: f64,
    }

    // Point constraints
    for i in 0..lines {
        let p1 = points[i];
        let p2 = points[i + 1];
        // C[i](0) = P[i]
        // C[i](1) = P[i + 1]
        constraints.push(Constraint {
            sum: vec![(1., Var { line: i, coe: 0 })],
            eq: p1,
        });
        constraints.push(Constraint {
            sum: vec![
                (1., Var { line: i, coe: 0 }),
                (1., Var { line: i, coe: 1 }),
                (1., Var { line: i, coe: 2 }),
                (1., Var { line: i, coe: 3 }),
            ],
            eq: p2,
        });
    }
    for i in 0..lines - 1 {
        //    C[i]'(1) = C[i+1]'(0)
        // b + 2c + 3d = f
        constraints.push(Constraint {
            sum: vec![
                (1., Var { line: i, coe: 1 }),
                (2., Var { line: i, coe: 2 }),
                (3., Var { line: i, coe: 3 }),
                (
                    -1.,
                    Var {
                        line: i + 1,
                        coe: 1,
                    },
                ),
            ],
            eq: 0.,
        });
        //   C[i]''(1) = C[i+1]''(0)
        // 2c + 6d = 2g
        constraints.push(Constraint {
            sum: vec![
                (2., Var { line: i, coe: 2 }),
                (6., Var { line: i, coe: 3 }),
                (
                    -2.,
                    Var {
                        line: i + 1,
                        coe: 2,
                    },
                ),
            ],
            eq: 0.,
        });
    }

    // Complete end conditions
    //    C[0]'(0) = 0, C[n-1]'(1) = 0
    constraints.push(Constraint {
        sum: vec![(1., Var { line: 0, coe: 1 })],
        eq: 0.,
    });
    constraints.push(Constraint {
        sum: vec![
            (
                1.,
                Var {
                    line: lines - 1,
                    coe: 1,
                },
            ),
            (
                2.,
                Var {
                    line: lines - 1,
                    coe: 2,
                },
            ),
            (
                3.,
                Var {
                    line: lines - 1,
                    coe: 3,
                },
            ),
        ],
        eq: 0.,
    });

    assert_eq!(constraints.len(), vars);
    let mut array = Array2::<f64>::zeros((vars, constraints.len()));
    let mut b = Array1::<f64>::zeros(constraints.len());
    for (i, Constraint { sum, eq }) in constraints.into_iter().enumerate() {
        b[i] = eq;
        for (m, Var { line, coe }) in sum {
            let j = line * 4 + coe as usize;
            array[(i, j)] = m;
        }
    }

    let x = array.solve(&b).unwrap();
    let mut ret = vec![];
    for i in 0..lines {
        ret.push(Poly {
            a: x[4 * i],
            b: x[4 * i + 1],
            c: x[4 * i + 2],
            d: x[4 * i + 3],
        });
    }

    ret
}

fn samples(lines: &[(Poly, Poly)], out: &mut Vec<FPoint>) {
    if lines.is_empty() {
        return;
    }
    out.clear();
    for (l1, l2) in lines {
        const N: usize = 100;
        for i in 0..N {
            let x = l1.get(i as f64 / (N - 1) as f64);
            let y = l2.get(i as f64 / (N - 1) as f64);
            out.push(FPoint::new(x as _, y as _));
        }
    }
}

pub fn main() {
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl3 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut points: Vec<sdl3::render::FPoint> = vec![
    ];
    let mut actual_points = vec![];

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(10, 10, 10 + i / 20));
        canvas.clear();
        let px = polyline(points.iter().map(|p| p.x as f64).collect::<Vec<_>>().as_slice());
        let py = polyline(points.iter().map(|p| p.y as f64).collect::<Vec<_>>().as_slice());
        let lines: Vec<(Poly, Poly)> = px.into_iter().zip(py).collect();
        samples(&lines, &mut actual_points);
        if points.len() == 2 {
            dbg!(&points, &actual_points, &lines);
        }
        canvas.set_draw_color(Color::WHITE);
        canvas.draw_lines(actual_points.as_slice()).unwrap();
        canvas.set_draw_color(Color::RED);
        for point in &points {
            canvas
                .draw_rect(sdl3::rect::Rect::new(
                    point.x as i32 - 2,
                    point.y as i32 - 2,
                    4,
                    4,
                ))
                .unwrap();
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown { x, y, .. } => {
                    println!("Adding point at {}, {}", x, y);
                    points.push(FPoint::new(x, y));
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
