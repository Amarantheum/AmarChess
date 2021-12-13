use chess::*;

use std::sync::{Arc, Mutex};
use std::cmp::{max, min};

use rayon::prelude::*;
use counts::*;
use negamax::*;

pub mod negamax;
pub mod counts;

fn evaluate_board(board: &Board, move_color: bool) -> i32 {
    let moves = MoveGen::new_legal(&board);
    if moves.len() == 0 {
        if board.checkers().to_size(0) == 0 {
            return 0;
        } else {
            return i32::MIN;
        }
    }
    let mut eval = get_piece_values(board, Color::White) - get_piece_values(board, Color::Black);
    let bishops = board.pieces(Piece::Bishop);
    if (bishops & board.color_combined(Color::White)).popcnt() == 2 {
        eval += 50;
    }
    if (bishops & board.color_combined(Color::Black)).popcnt() == 2 {
        eval += 50; 
    }
    if move_color {
        eval
    } else {
        -eval
    }
}

fn get_piece_values(board: &Board, color: Color) -> i32 {
    let mut val = 0;
    // queens
    val += (board.color_combined(color) & board.pieces(Piece::Queen)).popcnt() * 950;
    val += (board.color_combined(color) & board.pieces(Piece::Rook)).popcnt() * 560;
    val += (board.color_combined(color) & board.pieces(Piece::Bishop)).popcnt() * 330;
    val += (board.color_combined(color) & board.pieces(Piece::Knight)).popcnt() * 300;
    val += (board.color_combined(color) & board.pieces(Piece::Pawn)).popcnt() * 100;
    val as i32
}

pub fn find_best_move_iterative(board: Board, depth: usize, move_color: bool) -> ChessMove {
    fn find_best_move_sorted_moves(board: Board, depth: usize, move_color: bool, moves: Vec<ChessMove>) -> Vec<ChessMove> { 
        assert!(depth % 2 == 0);
        fn internal_fn(board: Board, depth: usize, mut alpha: i32, mut beta: i32, maximizing: bool, move_color: bool) -> i32 {
            if depth == 0 {
                let eval = evaluate_board(&board, move_color);
                return eval;
            }
            if maximizing {
                let moves = MoveGen::new_legal(&board);
                if moves.len() == 0 {
                    if board.checkers().to_size(0) == 0 {
                        return 0;
                    } else {
                        return i32::MIN;
                    }
                }
                let mut best_value = i32::MIN;
                for m in moves {
                    let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
                    best_value = max(best_value, calculation);
                    alpha = max(best_value, alpha);
                    if alpha >= beta {
                        break;
                    }
                }
                return best_value;
            } else {
                let moves = MoveGen::new_legal(&board);
                if moves.len() == 0 {
                    if board.checkers().to_size(0) == 0 {
                        return 0;
                    } else {
                        return i32::MAX;
                    }
                }
                let mut best_value = i32::MAX;
                for m in moves {
                    let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, true, move_color);
                    best_value = min(best_value, calculation);
                    beta = min(best_value, beta);
                    if alpha >= beta {
                        break;
                    }
                }
                return best_value;
            }
        }
        let mut alpha = i32::MIN;
        let beta = i32::MAX;
        let mut best = i32::MIN;
        let mut move_values = vec![];
        for m in moves {
            let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
            move_values.push((calculation, m));
            best = max(best, calculation);
            alpha = max(best, alpha);
            
        }
        move_values[..].sort_by(|a, b| std::cmp::Ordering::reverse(a.0.cmp(&b.0)));
        move_values.iter().map(|v| v.1).collect::<Vec<ChessMove>>()
    }
    let mut moves = MoveGen::new_legal(&board).collect::<Vec<ChessMove>>();
    let mut run_depth = 2;
    while run_depth <= depth {
        moves = find_best_move_sorted_moves(board, run_depth, move_color, moves);
        run_depth += 2;
    }
    moves.remove(0)
}

// move_color: true = white, false = black
pub fn find_best_move_single(board: Board, depth: usize, move_color: bool) -> ChessMove { 
    assert!(depth % 2 == 0);
    fn internal_fn(board: Board, depth: usize, mut alpha: i32, mut beta: i32, maximizing: bool, move_color: bool, mov: ChessMove) -> (i32, ChessMove) {
        if depth == 0 {
            let eval = evaluate_board(&board, move_color);
            return (eval, mov);
        }
        if maximizing {
            let moves = MoveGen::new_legal(&board);
            if moves.len() == 0 {
                if board.checkers().to_size(0) == 0 {
                    return (0, mov);
                } else {
                    return (i32::MIN, mov);
                }
            }
            let mut best_value = (i32::MIN, mov);
            for m in moves {
                let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color, m);
                best_value = (max(best_value.0, calculation.0), mov);
                alpha = max(best_value.0, alpha);
                if alpha >= beta {
                    break;
                }
            }
            return best_value;
        } else {
            let moves = MoveGen::new_legal(&board);
            if moves.len() == 0 {
                if board.checkers().to_size(0) == 0 {
                    return (0, mov);
                } else {
                    return (i32::MAX, mov);
                }
            }
            let mut best_value = (i32::MAX, mov);
            for m in moves {
                let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, true, move_color, m);
                best_value = (min(best_value.0, calculation.0), mov);
                beta = min(best_value.0, beta);
                if alpha >= beta {
                    break;
                }
            }
            return best_value;
        }
    }
    let moves = MoveGen::new_legal(&board);
    let mut best_value = (i32::MIN, moves.last().unwrap());
    let mut alpha = i32::MIN;
    let beta = i32::MAX;
    for m in MoveGen::new_legal(&board) {
        let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color, m);
        if calculation.0 > best_value.0 {
            best_value = (calculation.0, m);
        }
        alpha = max(best_value.0, alpha);
        
    }
    best_value.1
}

fn find_best_move_sorted_moves(board: Board, depth: usize, move_color: bool, moves: Vec<ChessMove>) -> Vec<ChessMove> { 
    assert!(depth % 2 == 0);
    fn internal_fn(board: Board, depth: usize, mut alpha: i32, mut beta: i32, maximizing: bool, move_color: bool) -> i32 {
        if depth == 0 {
            let eval = evaluate_board(&board, move_color);
            return eval;
        }
        if maximizing {
            let moves = MoveGen::new_legal(&board);
            if moves.len() == 0 {
                if board.checkers().to_size(0) == 0 {
                    return 0;
                } else {
                    return i32::MIN;
                }
            }
            let mut best_value = i32::MIN;
            for m in moves {
                let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
                best_value = max(best_value, calculation);
                alpha = max(best_value, alpha);
                if alpha >= beta {
                    break;
                }
            }
            return best_value;
        } else {
            let moves = MoveGen::new_legal(&board);
            if moves.len() == 0 {
                if board.checkers().to_size(0) == 0 {
                    return 0;
                } else {
                    return i32::MAX;
                }
            }
            let mut best_value = i32::MAX;
            for m in moves {
                let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, true, move_color);
                best_value = min(best_value, calculation);
                beta = min(best_value, beta);
                if alpha >= beta {
                    break;
                }
            }
            return best_value;
        }
    }
    let mut alpha = i32::MIN;
    let beta = i32::MAX;
    let mut best = i32::MIN;
    let mut move_values = vec![];
    for m in moves {
        let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
        move_values.push((calculation, m));
        best = max(best, calculation);
        alpha = max(best, alpha);
        
    }
    move_values[..].sort_by(|a, b| std::cmp::Ordering::reverse(a.0.cmp(&b.0)));
    move_values.iter().map(|v| v.1).collect::<Vec<ChessMove>>()
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use crate::engine::negamax::find_best_move_nega;

    use super::*;
    use std::time;
    use std::str::FromStr;
    use std::collections::HashMap;

    lazy_static!(
        static ref PUZZLES: HashMap<&'static str, &'static str> = {
            let mut map = HashMap::new();
            map.insert("r1b1kb1r/pppp1ppp/5q2/4n3/3KP3/2N3PN/PPP4P/R1BQ1B1R b kq - 0 1", "Bc5+");
            map.insert("r3k2r/ppp2Npp/1b5n/4p2b/2B1P2q/BQP2P2/P5PP/RN5K w kq - 1 0", "Bb5+");
            map.insert("r1b3kr/ppp1Bp1p/1b6/n2P4/2p3q1/2Q2N2/P4PPP/RN2R1K1 w - - 1 0", "Qxh8+");
            map.insert("r2n1rk1/1ppb2pp/1p1p4/3Ppq1n/2B3P1/2P4P/PP1N1P1K/R2Q1RN1 b - - 0 1", "Qxf2+");
            map.insert("3q1r1k/2p4p/1p1pBrp1/p2Pp3/2PnP3/5PP1/PP1Q2K1/5R1R w - - 1 0", "Rxh7+");
            map
        };
    );

    #[test]
    fn test_mate_single() {
        let board = Board::from_str("7k/8/8/8/8/8/5R2/1K4R1 w - - 0 1").unwrap();
        assert_eq!(find_best_move_single(board, 6, true), ChessMove::from_san(&board, "Rh2").unwrap());
    }

    #[test]
    fn test_take_queen_single() {
        let board = Board::from_str("5k2/8/3q4/8/8/8/8/1K1Q4 w - - 0 1").unwrap();
        assert_eq!(find_best_move_single(board, 4, true), ChessMove::from_san(&board, "Qxd6").unwrap());
    }

    #[test]
    fn test_mate_in_2() {
        let board = Board::from_str("r2qkb1r/pp2nppp/3p4/2pNN1B1/2BnP3/3P4/PPP2PPP/R2bK2R w KQkq - 1 0").unwrap();
        assert_eq!(find_best_move_single(board, 6, true), ChessMove::from_san(&board, "Nf6+").unwrap());
    }

    #[test]
    fn test_mate_in_2_2() {
        let board = Board::from_str("7k/1p4p1/p4b1p/3N3P/2p5/2rb4/PP2r3/K2R2R1 b - - 0 1").unwrap();
        assert_eq!(find_best_move_single(board, 4, false), ChessMove::from_san(&board, "Rc1+").unwrap());
    }
    #[test]
    fn test_win_piece() {
        let board = Board::from_str("r5k1/1R3bp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 1").unwrap();
        let best = find_best_move_single(board, 6, true);
        assert_eq!(best, ChessMove::from_san(&board, "Rxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_single(board, 4, false);
        assert_eq!(best, ChessMove::from_san(&board, "Kxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_single(board, 6, true);
        assert_eq!(best, ChessMove::from_san(&board, "Qb7+").unwrap());
    }

    #[test]
    fn test_win_piece_counts() {
        let board = Board::from_str("r5k1/1R3bp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 1").unwrap();
        let best = find_best_move_single_count(board, 6, true);
        assert_eq!(best, ChessMove::from_san(&board, "Rxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_single_count(board, 6, false);
        assert_eq!(best, ChessMove::from_san(&board, "Kxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_single_count(board, 6, true);
        assert_eq!(best, ChessMove::from_san(&board, "Qb7+").unwrap());
    }

    #[test]
    fn test_win_piece_counts_2() {
        let board = Board::from_str("r5k1/1R3bp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 1").unwrap();
        let best = find_best_move_single_count(board, 4, true);
        let best = find_best_move_iterative_count(board, 6, true);
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_single_count(board, 4, false);
        let best = find_best_move_iterative_count(board, 6, true);
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_single_count(board, 4, true);
        let best = find_best_move_iterative_count(board, 6, true);
    }

    #[test]
    fn test_win_piece_2() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        assert_eq!(find_best_move_single(board, 4, true), ChessMove::from_san(&board, "Qb7+").unwrap());
    }

    #[test]
    fn test_mate_in_2_inc() {
        let board = Board::from_str("7k/1p4p1/p4b1p/3N3P/2p5/2rb4/PP2r3/K2R2R1 b - - 0 1").unwrap();
        assert_eq!(find_best_move_iterative(board, 8, false), ChessMove::from_san(&board, "Rc1+").unwrap());
    }

    #[test]
    fn test_mate_in_2_inc_nega() {
        let board = Board::from_str("7k/1p4p1/p4b1p/3N3P/2p5/2rb4/PP2r3/K2R2R1 b - - 0 1").unwrap();
        assert_eq!(find_best_move_nega_iterative_transposition(board, 8), ChessMove::from_san(&board, "Rc1+").unwrap());
    }

    #[test]
    fn test_win_piece_2_iter() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        assert_eq!(find_best_move_iterative(board, 8, true), ChessMove::from_san(&board, "Qb7+").unwrap());
    }

    #[test]
    fn test_win_piece_cmp() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_single(board, 6, true), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("single: {}", start.elapsed().as_secs_f32());
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("nega: {}", start.elapsed().as_secs_f32());
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_iterative(board, 6, true), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("iter: {}", start.elapsed().as_secs_f32());
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("nega iter: {}", start.elapsed().as_secs_f32());
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative_transposition(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("nega iter transpose: {}", start.elapsed().as_secs_f32());
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative_transposition_ordering(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("nega iter transpose ordering: {}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_win_piece_cmp2() {
        let mut board = Board::default();
        let start = time::Instant::now();
        for _ in 0..10 {
            let m = find_best_move_nega_iterative_transposition(board, 6);
            board = board.make_move_new(m);
        }
        println!("{}", start.elapsed().as_secs_f32());
        let mut board = Board::default();
        let start = time::Instant::now();
        for _ in 0..10 {
            let m = find_best_move_nega_iterative_transposition_ordering(board, 6);
            board = board.make_move_new(m);
        }
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_eval() {
        assert_eq!(evaluate_board(&Board::default(), true), 0);
        assert_eq!(evaluate_board(&Board::default(), false), 0);
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        assert_eq!(evaluate_board(&board, true), -30);
        assert_eq!(evaluate_board(&board, false), 30);
    }

    #[test]
    fn test_win_piece_iterative() {
        let board = Board::from_str("r5k1/1R3bp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 1").unwrap();
        let best = find_best_move_iterative(board, 6, true);
        assert_eq!(best, ChessMove::from_san(&board, "Rxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_iterative(board, 6, false);
        assert_eq!(best, ChessMove::from_san(&board, "Kxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_iterative(board, 6, true);
        assert_eq!(best, ChessMove::from_san(&board, "Qb7+").unwrap());
    }

    #[test]
    fn test_win_piece_nega_iterative() {
        let board = Board::from_str("r5k1/1R3bp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 1").unwrap();
        let best = find_best_move_nega_iterative_transposition(board, 6);
        assert_eq!(best, ChessMove::from_san(&board, "Rxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_nega_iterative_transposition(board, 6);
        assert_eq!(best, ChessMove::from_san(&board, "Kxf7").unwrap());
        println!("bruh");
        let board = board.make_move_new(best);
        let best = find_best_move_nega_iterative_transposition(board, 6);
        assert_eq!(best, ChessMove::from_san(&board, "Qb7+").unwrap());
    }

    #[test]
    fn test_solve_puzzles() {
        let start = time::Instant::now();
        for (board, m) in &*PUZZLES {
            let tmp = time::Instant::now();
            let board = Board::from_str(board).unwrap();
            let mov = ChessMove::from_san(&board, m).unwrap();
            println!("board: {}", board);
            assert_eq!(find_best_move_nega_iterative_transposition(board, 6), mov);
            println!("took: {}", tmp.elapsed().as_secs_f32());
        }
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_solve_puzzles_ordering() {
        let start = time::Instant::now();
        for (board, m) in &*PUZZLES {
            let tmp = time::Instant::now();
            let board = Board::from_str(board).unwrap();
            let mov = ChessMove::from_san(&board, m).unwrap();
            println!("board: {}", board);
            assert_eq!(find_best_move_nega_iterative_transposition_ordering(board, 6), mov);
            println!("took: {}", tmp.elapsed().as_secs_f32());
        }
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_solve_puzzles_2() {
        for (board, m) in &*PUZZLES {
            let board = Board::from_str(board).unwrap();
            let mov = ChessMove::from_san(&board, m).unwrap();
            println!("board: {}", board);
            assert_eq!(find_best_move_nega(board, 6), mov);
        }
    }

    #[test]
    fn test_solve_puzzles_normal() {
        for (board, m) in &*PUZZLES {
            let board = Board::from_str(board).unwrap();
            let mov = ChessMove::from_san(&board, m).unwrap();
            println!("board: {}", board);
            let side = match board.side_to_move() {
                Color::Black => false,
                Color::White => true,
            };
            assert_eq!(find_best_move_iterative(board, 6, side), mov);
        }
    }
}