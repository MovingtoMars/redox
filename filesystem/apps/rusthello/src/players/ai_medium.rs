//use rand;
//use rand::Rng;

use reversi;
use players::Score;

const MOBILITY: u8 = 1;
//const RANDOMNESS: f32 = 1.0;



pub fn ai_eval(game: &reversi::Game, depth: u8) -> Score {

    match game.get_status() {
        reversi::Status::Running { current_turn } => {
            if depth == 0 {
                Score::Running(heavy_eval(game) as f32)
            } else {
                let mut best_score: Option<Score> = None;
                let mut num_moves: u8 = 0;
                let mut game_after_move = game.clone();

                for (row, &rows) in game.get_board().iter().enumerate() {
                    for (col, _) in rows.iter().enumerate() {
                        if game_after_move.make_move((row, col)) {

                            num_moves += 1;
                            let new_score = ai_eval(&game_after_move, depth - 1);
                            match best_score.clone() {
                                Some(old_score) => {
                                    if Score::is_better_for(new_score.clone(), old_score, current_turn) {
                                        best_score = Some(new_score);
                                    }
                                }
                                None => best_score = Some(new_score),
                            }
                            game_after_move = game.clone();

                        }
                    }
                }
                if let Some(score) = best_score {
                    if let Score::Running(val) = score {
                        return match current_turn {
                            reversi::Disk::Light => Score::Running(val + ( num_moves * MOBILITY ) as f32 ),
                            reversi::Disk::Dark  => Score::Running(val - ( num_moves * MOBILITY ) as f32 ),
                        }
                    } else {
                        return score;
                    }
                } else {
                    panic!("ai_eval produced no best_score!");
                }
            }
        }
        reversi::Status::Ended => {
            Score::EndGame(game.get_score_diff())
        }
    }
}



fn heavy_eval(game: &reversi::Game) -> i16 {
    const CORNER_BONUS: i16 = 15;
    const ODD_MALUS: i16 = 3;
    const EVEN_BONUS: i16 = 3;
    const ODD_CORNER_MALUS: i16 = 10;
    const EVEN_CORNER_BONUS: i16 = 5;
    const FIXED_BONUS: i16 = 3;

    const SIDES: [( (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize) ); 4] = [
        ( (0,0), (0,1), (1,1), (0,2), (2,2), (1,0), (2,0) ), // NW corner
        ( (0,7), (1,7), (1,6), (2,7), (2,5), (0,6), (0,5) ), // NE corner
        ( (7,0), (6,0), (6,1), (5,0), (5,2), (7,1), (7,2) ), // SW corner
        ( (7,7), (6,7), (6,6), (5,7), (5,5), (7,6), (7,5) ), // SE corner
        ];

    let mut score: i16 = 0;

    for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in SIDES.iter() {

        if let reversi::Cell::Taken { disk } = game.get_cell(corner) {
            match disk {
                reversi::Disk::Light => {
                    score += CORNER_BONUS;
                    if let reversi::Cell::Taken { disk: reversi::Disk::Light } = game.get_cell(odd) {
                        score += FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Disk::Light } = game.get_cell(even) {
                            score += FIXED_BONUS;
                        }
                    }
                    if let reversi::Cell::Taken { disk: reversi::Disk::Light } = game.get_cell(counter_odd) {
                        score += FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Disk::Light } = game.get_cell(counter_even) {
                            score += FIXED_BONUS;
                        }
                    }
                }
                reversi::Disk::Dark => {
                    score -= CORNER_BONUS;
                    if let reversi::Cell::Taken { disk: reversi::Disk::Dark } = game.get_cell(odd) {
                        score -= FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Disk::Dark } = game.get_cell(even) {
                            score -= FIXED_BONUS;
                        }
                    }
                    if let reversi::Cell::Taken { disk: reversi::Disk::Dark } = game.get_cell(counter_odd) {
                        score -= FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Disk::Dark } = game.get_cell(counter_even) {
                            score -= FIXED_BONUS;
                        }
                    }
                }
            }

        } else {

            if let reversi::Cell::Taken { disk } = game.get_cell(odd) {
                score += match disk {
                    reversi::Disk::Light => -ODD_MALUS,
                    reversi::Disk::Dark  =>  ODD_MALUS,
                }
            } else if let reversi::Cell::Taken { disk } = game.get_cell(even) {
                score += match disk {
                    reversi::Disk::Light => EVEN_BONUS,
                    reversi::Disk::Dark  => -EVEN_BONUS,
                }
            }

            if let reversi::Cell::Taken { disk } = game.get_cell(counter_odd) {
                score += match disk {
                    reversi::Disk::Light => -ODD_MALUS,
                    reversi::Disk::Dark  =>  ODD_MALUS,
                }
            } else if let reversi::Cell::Taken { disk } = game.get_cell(counter_even) {
                score += match disk {
                    reversi::Disk::Light =>  EVEN_BONUS,
                    reversi::Disk::Dark  => -EVEN_BONUS,
                }
            }

            if let reversi::Cell::Taken { disk } = game.get_cell(odd_corner) {
                score += match disk {
                    reversi::Disk::Light => -ODD_CORNER_MALUS,
                    reversi::Disk::Dark  =>  ODD_CORNER_MALUS,
                }

            } else if let reversi::Cell::Taken { disk } = game.get_cell(even_corner) {
                score += match disk {
                    reversi::Disk::Light =>  EVEN_CORNER_BONUS,
                    reversi::Disk::Dark  => -EVEN_CORNER_BONUS,
                }
            }
        }
    }

    score
}
