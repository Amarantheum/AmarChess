use chess::*;
use fltk::{enums::Event, frame::Frame, image::SvgImage, app::Sender};
use fltk::*;
use fltk::prelude::*;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref DARK_SQUARE: enums::Color = enums::Color::from_hex(u32::from_str_radix("9c504c", 16).unwrap());
    static ref LIGHT_SQUARE: enums::Color = enums::Color::from_hex(u32::from_str_radix("ffffff", 16).unwrap());
    static ref HIGHLIGHT_SQUARE: enums::Color = enums::Color::from_hex(u32::from_str_radix("bae11e", 16).unwrap());
}

#[derive(Clone)]
pub struct Square {
    pub frame: Frame,
    pub color: enums::Color,
    pub img: Option<SvgImage>,
    pub pos: (i32, i32),
    pub square: chess::Square,
    pub mouse_drag_frame: Arc<Mutex<Frame>>,
}

impl Square {
    pub fn new(mut frame: Frame, color: enums::Color, img: Option<SvgImage>, sender: Arc<Mutex<Sender<SquareMessage>>>, pos: (i32, i32), scale: i32, drag_frame: Arc<Mutex<Frame>>) -> Self {
        frame.set_frame(enums::FrameType::FlatBox);
        frame.set_label_size(0);
        frame.set_color(color);
        let square = chess::Square::make_square(Rank::from_index(pos.1 as usize), File::from_index(pos.0 as usize));
        let mut out = Self {
            frame,
            color,
            img,
            pos,
            square,
            mouse_drag_frame: drag_frame,
        };
        out.update_scale(scale);
        out.set_click_event(sender);
        out
    }

    pub fn update_scale(&mut self, scale: i32) {
        self.frame.set_pos(self.pos.0 * scale, (7 - self.pos.1) * scale);
        self.frame.set_size(scale, scale);
        self.set_image();
    }

    pub fn update_image(&mut self) {
        let board = *super::BOARD.lock().unwrap();
        let color = match board.color_on(self.square) {
            None => {
                self.img = None;
                self.set_image();
                return;
            },
            Some(c) => match c {
                chess::Color::Black => "black",
                chess::Color::White => "white",
            },
        };
        let piece = match board.piece_on(self.square) {
            None => {
                self.img = None;
                self.set_image();
                return;
            },
            Some(p) => super::match_piece(p),
        };
        let key = color.to_string() + "_" + piece;
        self.img = (*super::PIECE_IMAGES).get(key.as_str()).cloned();
        self.set_image();
    }

    pub fn setup_board_squares(s: Arc<Mutex<Sender<SquareMessage>>>, scale: i32, drag_frame: Arc<Mutex<Frame>>) -> Vec<Square> {
        let mut squares: Vec<Square> = vec![];
        for r in 0..8 {
            for c in 0..8 {
                let square_frame = Frame::new(0, 0, 0, 0, "");
                let drag_frame_clone = Arc::clone(&drag_frame);
    
                let color = if (r + c) % 2 == 0 {
                    *DARK_SQUARE
                } else {
                    *LIGHT_SQUARE
                };
                let mut s = Square::new(square_frame, color, None, Arc::clone(&s), (c, r), scale, drag_frame_clone);
                s.update_image();
                squares.push(s);
            }
        }
        squares
    }

    fn set_image(&mut self) {
        if let Some(i) = &self.img {
            self.frame.set_image(Some({
                let mut img = i.clone();
                img.scale(self.frame.w(), self.frame.h(), false, true);
                img
            }));
        } else {
            self.frame.set_image::<SvgImage>(None);
        }
        self.frame.redraw();
    }

    fn set_click_event(&mut self, sender: Arc<Mutex<Sender<SquareMessage>>>) {
        let square = self.square.clone();
        self.frame.handle(move |_, e| {
            println!("{:?}", e);
            match e {
                Event::Push => {
                    sender.lock().unwrap().send(SquareMessage::Click(square));
                },
                Event::Drag => {
                    sender.lock().unwrap().send(SquareMessage::Drag(square));
                }
                Event::Released => {
                    sender.lock().unwrap().send(SquareMessage::Released(square));
                }
                _ => (),
            }
            true
        })
    }

    pub fn reset_color(&mut self) {
        self.frame.set_color(self.color);
        self.frame.redraw();
    }

    pub fn highlight(&mut self) {
        self.frame.set_color(*HIGHLIGHT_SQUARE);
        self.frame.redraw();
    }
}

pub enum SquareMessage {
    Click(chess::Square),
    Drag(chess::Square),
    Released(chess::Square),
}