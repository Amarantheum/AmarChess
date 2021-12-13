use chess::*;

use std::sync::{Arc, Mutex};
use std::cmp::{max, min};

use rayon::prelude::*;
use super::evaluate_board;
// move_color: true = white, false = black
pub fn find_best_move_single_count(board: Board, depth: usize, move_color: bool) -> ChessMove { 
    assert!(depth % 2 == 0);
    fn internal_fn(board: Board, depth: usize, mut alpha: i32, mut beta: i32, maximizing: bool, move_color: bool) -> (i32, u32) {
        if depth == 0 {
            let eval = evaluate_board(&board, move_color);
            return (eval, 1);
        }
        let mut count = 1;
        if maximizing {
            let moves = MoveGen::new_legal(&board);
            if moves.len() == 0 {
                if board.checkers().to_size(0) == 0 {
                    return (0, 1);
                } else {
                    return (i32::MIN, 1);
                }
            }
            let mut best_value = i32::MIN;
            for m in moves {
                let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
                count += calculation.1;
                best_value = max(best_value, calculation.0);
                alpha = max(best_value, alpha);
                if alpha >= beta {
                    break;
                }
            }
            return (best_value, count);
        } else {
            let moves = MoveGen::new_legal(&board);
            if moves.len() == 0 {
                if board.checkers().to_size(0) == 0 {
                    return (0, 1);
                } else {
                    return (i32::MAX, 1);
                }
            }
            let mut best_value = i32::MAX;
            for m in moves {
                let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, true, move_color);
                count += calculation.1;
                best_value = min(best_value, calculation.0);
                beta = min(best_value, beta);
                if alpha >= beta {
                    break;
                }
            }
            return (best_value, count);
        }
    }
    let moves = MoveGen::new_legal(&board);
    let mut best_value = (i32::MIN, moves.last().unwrap());
    let mut alpha = i32::MIN;
    let beta = i32::MAX;
    let mut count = 1;
    for m in MoveGen::new_legal(&board) {
        let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
        count += calculation.1;
        if calculation.0 > best_value.0 {
            best_value = (calculation.0, m);
        }
        alpha = max(best_value.0, alpha);
        
    }
    println!("count: {}", count);
    best_value.1
}

pub fn find_best_move_iterative_count(board: Board, depth: usize, move_color: bool) -> ChessMove {
    fn find_best_move_sorted_moves(board: Board, depth: usize, move_color: bool, moves: Vec<ChessMove>) -> Vec<ChessMove> { 
        assert!(depth % 2 == 0);
        fn internal_fn(board: Board, depth: usize, mut alpha: i32, mut beta: i32, maximizing: bool, move_color: bool) -> (i32, u32) {
            let mut count = 1;
            if depth == 0 {
                let eval = evaluate_board(&board, move_color);
                return (eval, count);
            }
            if maximizing {
                let moves = MoveGen::new_legal(&board);
                if moves.len() == 0 {
                    if board.checkers().to_size(0) == 0 {
                        return (0, count);
                    } else {
                        return (i32::MIN, count);
                    }
                }
                let mut best_value = i32::MIN;
                for m in moves {
                    let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
                    count += calculation.1;
                    best_value = max(best_value, calculation.0);
                    alpha = max(best_value, alpha);
                    if alpha >= beta {
                        break;
                    }
                }
                return (best_value, count);
            } else {
                let moves = MoveGen::new_legal(&board);
                if moves.len() == 0 {
                    if board.checkers().to_size(0) == 0 {
                        return (0, count);
                    } else {
                        return (i32::MAX, count);
                    }
                }
                let mut best_value = i32::MAX;
                for m in moves {
                    let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, true, move_color);
                    count += calculation.1;
                    best_value = min(best_value, calculation.0);
                    beta = min(best_value, beta);
                    if alpha >= beta {
                        break;
                    }
                }
                return (best_value, count);
            }
        }
        let mut alpha = i32::MIN;
        let beta = i32::MAX;
        let mut best = i32::MIN;
        let mut move_values = vec![];
        let mut count = 1;
        for m in moves {
            let calculation = internal_fn(board.make_move_new(m), depth - 1, alpha, beta, false, move_color);
            count += calculation.1;
            move_values.push((calculation.0, m));
            best = max(best, calculation.0);
            alpha = max(best, alpha);
            
        }
        println!("count: {}", count);
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