use chess::*;

use std::sync::{Arc, Mutex};
use std::cmp::{max, min};
use std::collections::HashMap;

fn evaluate_board(board: &Board) -> i32 {
    let mut eval = get_piece_values(board, Color::White) - get_piece_values(board, Color::Black);
    let bishops = board.pieces(Piece::Bishop);
    if (bishops & board.color_combined(Color::White)).popcnt() == 2 {
        eval += 50;
    }
    if (bishops & board.color_combined(Color::Black)).popcnt() == 2 {
        eval -= 50; 
    }
    eval
}

fn evaluate_board_improved(board: &Board) -> i32 {
    let eg_factor = (board.combined() ^ board.pieces(Piece::Pawn)).popcnt(); // 0 - 16 (or more)
    let mut eval = get_piece_values(board, Color::White) - get_piece_values(board, Color::Black);
    let bishops = board.pieces(Piece::Bishop);
    if (bishops & board.color_combined(Color::White)).popcnt() == 2 {
        eval += 50;
    }
    if (bishops & board.color_combined(Color::Black)).popcnt() == 2 {
        eval -= 50; 
    }
    let pawns = board.pieces(Piece::Pawn);
    let white_pawns = pawns & board.color_combined(Color::White);
    let black_pawns = pawns & board.color_combined(Color::Black);
    let mut white_pawn_bytes = vec![];
    let mut white_pawns_combined = 0;
    for i in 2..8 {
        let tmp = (white_pawns.0 >> 8 * i & 255) as u8;
        white_pawns_combined &= tmp;
        white_pawn_bytes.push(tmp)
    }

    let mut black_pawn_bytes = vec![];
    for i in 2..8 {
        black_pawn_bytes.push((black_pawns.0 >> 8 * i & 255) as u8)
    }
    
    let row_7 = (white_pawns.0 >> 56 & 255) as u8;
    let row_6 = (white_pawns.0 >> 48 & 255) as u8;
    let row_5 = (white_pawns.0 >> 40 & 255) as u8;
    let row_4 = (white_pawns.0 >> 32 & 255) as u8;
    let row_3 = (white_pawns.0 >> 24 & 255) as u8;
    let row_2 = (white_pawns.0 >> 16 & 255) as u8;
    eval
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

pub fn negamax(board: Board, depth: i8, mut alpha: i32, beta: i32, color: i32) -> i32 {
    let moves = MoveGen::new_legal(&board);
    if moves.len() == 0 {
        if board.checkers().popcnt() == 0 {
            return 0;
        } else {
            return -i32::MAX;
        }
    }
    if depth == 0 {
        return color * evaluate_board(&board)
    }
    
    let mut value = -i32::MAX;
    for m in moves {
        value = max(value, -negamax(board.make_move_new(m), depth - 1, -beta, -alpha, -color));
        alpha = max(alpha, value);
        if alpha >= beta {
            break;
        }
    }
    value
}

pub fn negamax_transposition(board: &Board, depth: i8, mut alpha: i32, beta: i32, color: i32, table: &mut HashMap<u64, (i32, i8)>) -> i32 {
    let moves = MoveGen::new_legal(&board);
    if moves.len() == 0 {
        let eval;
        if board.checkers().popcnt() == 0 {
            eval = 0;
        } else {
            eval = -i32::MAX;
        }
        table.entry(board.get_hash()).or_insert((eval, 0));
        return eval;
    }
    if depth == 0 {
        return color * evaluate_board(&board)
    }
    
    let mut value = -i32::MAX;
    for m in moves {
        let new = board.make_move_new(m);
        let hash = new.get_hash();
        let eval;
        match table.get(&hash) {
            None => {
                eval = -negamax_transposition(&new, depth - 1, -beta, -alpha, -color, table);
                table.insert(hash, (eval, depth));
            },
            Some(vv) => {
                if vv.1 >= depth {
                    eval = vv.0;
                } else {
                    eval = -negamax_transposition(&new, depth - 1, -beta, -alpha, -color, table);
                    table.insert(hash, (eval, depth));
                }
            },
        }
        value = max(value, eval);
        alpha = max(alpha, value);
        if alpha >= beta {
            break;
        }
    }
    value
}

pub fn negamax_transposition_ordering(board: &Board, depth: i8, mut alpha: i32, beta: i32, color: i32, table: &mut HashMap<u64, (i32, i8)>) -> i32 {
    let mut moves = MoveGen::new_legal(&board);
    if moves.len() == 0 {
        let eval;
        if board.checkers().popcnt() == 0 {
            eval = 0;
        } else {
            eval = -i32::MAX;
        }
        table.entry(board.get_hash()).or_insert((eval, 0));
        return eval;
    }
    if depth == 0 {
        return color * evaluate_board(&board)
    }

    let pieces = if color == 1 {
        board.color_combined(Color::Black)
    } else {
        board.color_combined(Color::White)
    };
    
    let mut value = -i32::MAX;
    moves.set_iterator_mask(*pieces);
    for _ in 0..2 {
        while let Some(m) = moves.next() {
            let new = board.make_move_new(m);
            let hash = new.get_hash();
            let eval;
            match table.get(&hash) {
                None => {
                    eval = -negamax_transposition_ordering(&new, depth - 1, -beta, -alpha, -color, table);
                    table.insert(hash, (eval, depth));
                },
                Some(vv) => {
                    if vv.1 >= depth {
                        eval = vv.0;
                    } else {
                        eval = -negamax_transposition_ordering(&new, depth - 1, -beta, -alpha, -color, table);
                        table.insert(hash, (eval, depth));
                    }
                },
            }
            value = max(value, eval);
            alpha = max(alpha, value);
            if alpha >= beta {
                return value;
            }
        }
        moves.set_iterator_mask(!EMPTY);
    }
    value
}

pub fn find_best_move_nega(board: Board, depth: i8) -> ChessMove {
    let color = match board.side_to_move() {
        Color::Black => -1,
        Color::White => 1,
    };
    let mut moves = MoveGen::new_legal(&board);
    let mut alpha = -i32::MAX;
    let beta = i32::MAX;
    let mut best_move = moves.next().unwrap();
    let mut best_value = -negamax(board.make_move_new(best_move), depth - 1, -beta, -alpha, -color);
    for m in moves {
        let calc = -negamax(board.make_move_new(m), depth - 1, -beta, -alpha, -color);
        if best_value < calc {
            best_value = calc;
            best_move = m;
        }
        alpha = max(alpha, best_value);

        if alpha >= beta {
            break;
        }
    }
    best_move
}

pub fn find_best_move_nega_transposition(board: Board, depth: i8) -> ChessMove {
    let color = match board.side_to_move() {
        Color::Black => -1,
        Color::White => 1,
    };
    let mut moves = MoveGen::new_legal(&board);
    let mut alpha = -i32::MAX;
    let beta = i32::MAX;
    let mut best_move = moves.next().unwrap();
    let mut table = HashMap::new();
    let mut best_value = -negamax_transposition(&board.make_move_new(best_move), depth - 1, -beta, -alpha, -color, &mut table);
    for m in moves {
        let calc = -negamax_transposition(&board.make_move_new(m), depth - 1, -beta, -alpha, -color, &mut table);
        if best_value < calc {
            best_value = calc;
            best_move = m;
        }
        alpha = max(alpha, best_value);

        if alpha >= beta {
            break;
        }
    }
    best_move
}

pub fn find_best_move_nega_moves(board: Board, depth: i8, moves: Vec<ChessMove>) -> Vec<ChessMove> {
    let color = match board.side_to_move() {
        Color::Black => -1,
        Color::White => 1,
    };
    let mut alpha = -i32::MAX;
    let beta = i32::MAX;
    let first_move = moves[0];
    alpha = -negamax(board.make_move_new(first_move), depth - 1, -beta, -alpha, -color);
    let mut move_values = vec![];
    move_values.push((first_move, alpha));
    let mut best_value = alpha;
    for m in moves {
        let calc = -negamax(board.make_move_new(m), depth - 1, -beta, -alpha, -color);
        if best_value < calc {
            best_value = calc;
        }
        move_values.push((m, calc));
        alpha = max(alpha, best_value);

        if alpha >= beta {
            break;
        }
    }
    move_values[..].sort_by(|a, b| std::cmp::Ordering::reverse(a.1.cmp(&b.1)));
    move_values.iter().map(|v| v.0).collect::<Vec<ChessMove>>()
}



pub fn find_best_move_nega_iterative(board: Board, depth: i8) -> ChessMove {
    let mut moves = MoveGen::new_legal(&board).collect::<Vec<ChessMove>>();
    let mut run_depth = 2;
    while run_depth <= depth {
        moves = find_best_move_nega_moves(board, run_depth, moves);
        run_depth += 2;
    }
    moves.remove(0)
}

pub fn find_best_move_nega_moves_transposition(board: &Board, depth: i8, moves: Vec<ChessMove>) -> Vec<ChessMove> {
    let color = match board.side_to_move() {
        Color::Black => -1,
        Color::White => 1,
    };
    let mut table = HashMap::new();
    let mut alpha = -i32::MAX;
    let beta = i32::MAX;
    let first_move = moves[0];
    alpha = -negamax_transposition(&board.make_move_new(first_move), depth - 1, -beta, -alpha, -color, &mut table);
    let mut move_values = vec![];
    move_values.push((first_move, alpha));
    let mut best_value = alpha;
    for m in moves {
        let calc = -negamax_transposition(&board.make_move_new(m), depth - 1, -beta, -alpha, -color, &mut table);
        if best_value < calc {
            best_value = calc;
        }
        move_values.push((m, calc));
        alpha = max(alpha, best_value);

        if alpha >= beta {
            break;
        }
    }
    move_values[..].sort_by(|a, b| std::cmp::Ordering::reverse(a.1.cmp(&b.1)));
    move_values.iter().map(|v| v.0).collect::<Vec<ChessMove>>()
}

pub fn find_best_move_nega_iterative_transposition(board: Board, depth: i8) -> ChessMove {
    let mut moves = MoveGen::new_legal(&board).collect::<Vec<ChessMove>>();
    let mut run_depth = 2;
    while run_depth <= depth {
        moves = find_best_move_nega_moves_transposition(&board, run_depth, moves);
        run_depth += 2;
    }
    moves.remove(0)
}

pub fn find_best_move_nega_moves_transposition_ordering(board: &Board, depth: i8, moves: Vec<ChessMove>) -> Vec<ChessMove> {
    let color = match board.side_to_move() {
        Color::Black => -1,
        Color::White => 1,
    };
    let mut table = HashMap::new();
    let mut alpha = -i32::MAX;
    let beta = i32::MAX;
    let first_move = moves[0];
    alpha = -negamax_transposition_ordering(&board.make_move_new(first_move), depth - 1, -beta, -alpha, -color, &mut table);
    let mut move_values = vec![];
    move_values.push((first_move, alpha));
    let mut best_value = alpha;
    for m in moves {
        let calc = -negamax_transposition_ordering(&board.make_move_new(m), depth - 1, -beta, -alpha, -color, &mut table);
        if best_value < calc {
            best_value = calc;
        }
        move_values.push((m, calc));
        alpha = max(alpha, best_value);

        if alpha >= beta {
            break;
        }
    }
    move_values[..].sort_by(|a, b| std::cmp::Ordering::reverse(a.1.cmp(&b.1)));
    move_values.iter().map(|v| v.0).collect::<Vec<ChessMove>>()
}

pub fn find_best_move_nega_iterative_transposition_ordering(board: Board, depth: i8) -> ChessMove {
    let mut moves = MoveGen::new_legal(&board).collect::<Vec<ChessMove>>();
    let mut run_depth = 2;
    while run_depth <= depth {
        moves = find_best_move_nega_moves_transposition_ordering(&board, run_depth, moves);
        run_depth += 2;
    }
    moves.remove(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time;
    use std::str::FromStr;

    #[test]
    fn test_win_piece_cmp() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_win_piece_cmp_2() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let moves = MoveGen::new_legal(&board).collect::<Vec<ChessMove>>();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_moves(board, 6, moves)[0], ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_win_piece_cmp_3() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }
    #[test]
    fn test_win_piece_cmp_4() {
        let board = Board::from_str("r7/5kp1/3p3p/2q2p2/p1P1pP2/4P1P1/1Q1N1K1P/8 w - - 0 2").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative_transposition(board, 6), ChessMove::from_san(&board, "Qb7+").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_win_piece_cmp_6() {
        let board = Board::from_str("3qk3/8/8/8/8/8/8/3QK3 w - - 0 1").unwrap();
        let start = time::Instant::now();
        find_best_move_nega_transposition(board, 6);
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_mate_in_1() {
        let board = Board::from_str("1n2kbnr/3ppppp/2b1r3/8/3qP3/1Q3PP1/3N3P/R1B1KBNR w Kk - 0 3").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative_transposition(board, 6), ChessMove::from_san(&board, "Qxb8#").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_mate_in_1_black() {
        let board = Board::from_str("2k3r1/5r2/8/8/8/8/8/7K b - - 0 1").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative_transposition(board, 6), ChessMove::from_san(&board, "Rh7#").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_eval_speed() {
        let board = Board::from_str("2k3r1/5r2/8/8/8/8/8/7K b - - 0 1").unwrap();
        let start = time::Instant::now();
        assert_eq!(find_best_move_nega_iterative_transposition(board, 6), ChessMove::from_san(&board, "Rh7#").unwrap());
        println!("{}", start.elapsed().as_secs_f32());
    }

    #[test]
    fn test_ordering() {
        let board = Board::default();
        let mut table = HashMap::new();
        negamax_transposition_ordering(&board, 2, -i32::MAX, i32::MAX, 1, &mut table);
    }
}