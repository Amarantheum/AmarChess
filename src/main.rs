use chess::*;
use fltk::{button::Button, enums::Event, frame::Frame, image::SvgImage, app::Sender};
use fltk::*;
use fltk::prelude::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::str::FromStr;

use crate::engine::{negamax::*};
use crate::engine::*;
use ui_square::{Square, SquareMessage};

mod engine;
mod ui_square;

lazy_static::lazy_static! {
    pub static ref PIECE_IMAGES: HashMap<String, SvgImage> = get_piece_images();
    pub static ref BOARD: Mutex<Board> = Mutex::new(Board::default());
}

fn main() { 
    rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();  
    
    let app = app::App::default();
    let mut wind = window::Window::default()
        .with_size(640, 480)
        .center_screen()
        .with_label("AmarChess");

    wind.set_color(enums::Color::White);

    let square_size = if wind.height() < wind.width() {
        wind.height() / 8
    } else {
        wind.width() / 8
    };

    let mut drag_frame = Frame::new(0,0, square_size, square_size, "drag square");
    drag_frame.set_label_size(0);
    drag_frame.hide();
    let drag_frame = Arc::new(Mutex::new(drag_frame));

    let (s, r) = app::channel::<SquareMessage>();
    let s = Arc::new(Mutex::new(s));
    let squares = Square::setup_board_squares(s, square_size, Arc::clone(&drag_frame));
    

    let button_squares_ref = Arc::new(Mutex::new(squares));
    let squares = Arc::clone(&button_squares_ref);
    let squares_app = Arc::clone(&button_squares_ref);

    let mut button = Button::new(500,0,20,20, "move");
    button.set_callback(move |b| {
        let board = (*BOARD.lock().unwrap()).clone();
        let m = find_best_move_nega_iterative_transposition_ordering(board, 6);
        let new_board = board.make_move_new(m);
        *BOARD.lock().unwrap() = new_board;
        for square in &mut *button_squares_ref.lock().unwrap() {
            square.update_image();
        }
    });
    
    wind.make_resizable(true);
    wind.end();
    wind.show();

    wind.handle(move |w, e| {
        match e {
            Event::Resize => {
                let scale = if w.height() < w.width() {
                    w.height() / 8
                } else {
                    w.width() / 8
                };
                for square in &mut *squares.lock().unwrap() {
                    square.update_scale(scale);
                }
            }
            _ => ()
        }
        true
    });

    let mut selected: Option<chess::Square> = None;
    while app.wait() {
        match r.recv() {
            None => {
                println!("{:?}", app::get_mouse())
            },
            Some(v) => match v {
                SquareMessage::Click(s) => {
                    match app::event_mouse_button() {
                        app::MouseButton::Left => {
                            for square in &mut *squares_app.lock().unwrap() {
                                square.reset_color();
                            }
                            let board = BOARD.lock().unwrap();
                            if let Some(ss) = selected {
                                println!("selected: {}", ss);
                                let m = ChessMove::new(ss, s, None);
                                if board.legal(m) {
                                    let new_board = board.make_move_new(m);
                                    drop(board);
                                    *BOARD.lock().unwrap() = new_board;
                                    println!("move: {}", s);
                                    for square in &mut *squares_app.lock().unwrap() {
                                        square.update_image();
                                    }
                                    selected = None;
                                    continue;
                                }
                            }
                            let col = match board.color_on(s) {
                                None => continue,
                                Some(c) => c,
                            };
                            
                            if board.side_to_move() == col {
                                selected = Some(s);
                                let move_board = get_move_bitboard(&col, &board, s);
                                let move_iter = move_board.into_iter().filter(|ss| {
                                    board.legal(ChessMove::new(s, *ss, None))
                                }).collect::<Vec<chess::Square>>();
                                drop(board);
                                let squares = &mut *squares_app.lock().unwrap();
                                let mut square_iter = squares.iter_mut();
                                for v in move_iter {
                                    let mut next = square_iter.next().unwrap();
                                    while next.square != v {
                                        next = square_iter.next().unwrap();
                                    }
                                    next.highlight();
                                }

                            } else {
                                continue;
                            }
                        },
                        app::MouseButton::Right => {

                        },
                        _ => continue,
                    }
                },
                SquareMessage::Drag(s) => {
                    match app::event_mouse_button() {
                        app::MouseButton::Left => {
                            println!("bruhh");
                        },
                        app::MouseButton::Right => {

                        },
                        _ => continue,
                    }
                },
                SquareMessage::Released(s) => {
                    match app::event_mouse_button() {
                        app::MouseButton::Left => {
                            println!("bruhh");
                        },
                        app::MouseButton::Right => {

                        },
                        _ => continue,
                    }
                },
                _ => unimplemented!("unimplemented event")
            }
        }
    }

    app.run().unwrap();
}

fn match_piece<'a>(p: Piece) -> &'a str {
    match p {
        Piece::King => "king",
        Piece::Queen => "queen",
        Piece::Bishop => "bishop",
        Piece::Knight => "knight",
        Piece::Rook => "rook",
        Piece::Pawn => "pawn",
    }
}

fn get_piece_images() -> HashMap<String, SvgImage> {
    let piece_list = vec!["white_king", "white_queen", "white_bishop", "white_knight", "white_rook", "white_pawn", "black_king", "black_queen", "black_bishop", "black_knight", "black_rook", "black_pawn"];
    let mut piece_map = HashMap::new();
    for piece in piece_list {
        piece_map.insert(piece.to_string(), SvgImage::load("./pieces/".to_string() + piece + ".svg").unwrap());
    }
    piece_map
}

fn get_move_bitboard(col: &Color, board: &Board, s: chess::Square) -> BitBoard {
    let op_col = match col {
        Color::Black => Color::White,
        Color::White => Color::Black,
    };
    match board.piece_on(s).unwrap() {
        Piece::Bishop => get_bishop_moves(s, *board.color_combined(op_col)),
        Piece::Rook => get_rook_moves(s, *board.color_combined(op_col)),
        Piece::Knight => get_knight_moves(s),
        Piece::King => {
            get_king_moves(s) | BitBoard::new((1 << 2) + (1 << 6) + (1 << 58) + (1 << 62))
        },
        Piece::Queen => {
            let blockers = board.color_combined(op_col);
            get_bishop_moves(s, *blockers) | get_rook_moves(s, *blockers)
        },
        Piece::Pawn => {
            get_pawn_moves(s, *col, *board.color_combined(op_col))
        },
    }
}


