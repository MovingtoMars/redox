//! A simple Reversi game written in Rust with love
//! by Enrico Ghiorzi

//extern crate rand;

// Import modules
mod reversi;
mod interface;
mod players;



pub fn main() {
    // Main intro
    println!("{}", interface::INTRO);

    loop {
        println!("{}", interface::MAIN_MENU);

        match interface::input_main_menu() {
            // Runs the game
            interface::UserCommand::NewGame => play_game(),
            // Prints help message
            interface::UserCommand::Help => println!("{}", interface::HELP),
            // Quit RUSThello
            interface::UserCommand::Quit => break,
            _ => panic!("Main got a user command it shouldn't have got!"),
        }
    }
}



fn play_game() {

    // Get the two players
    println!("{}", interface::NEW_PLAYER_MENU);
    let dark = match interface::new_player(reversi::Disk::Dark) {
        None => return,
        Some(player) => player,
    };
    let light = match interface::new_player(reversi::Disk::Light) {
        None => return,
        Some(player) => player,
    };

    // Create a new game
    let mut game = reversi::Game::new();
    let mut hystory: Vec<reversi::Game> = Vec::new();

    println!("{}", interface::COMMANDS_INFO);

    // Draw the current board and game info
    interface::draw_board(&game);

    // Proceed with turn after turn till the game ends
    'turn: while let reversi::Status::Running { current_turn } = game.get_status() {

        // If the game is running, get the coordinates of the new move from the right player
        let action = match current_turn {
            reversi::Disk::Light => light.make_move(&game),
            reversi::Disk::Dark  =>  dark.make_move(&game),
        };

        match action {
            // If the new move is valid, perform it; otherwise panic
            // Player's make_move method is responsible for returning a legal move
            // so the program should never print this message unless something goes horribly wrong
            interface::UserCommand::Move(row, col) => {

                if game.check_move((row, col)) {
                    hystory.push(game.clone());
                    game.make_move((row, col));
                    interface::draw_board(&game);
                } else {
                    panic!("Invalid move sent to main::game!");
                }
            }

            // Manage hystory
            interface::UserCommand::Undo => {
                let mut recovery: Vec<reversi::Game> = Vec::new();

                while let Some(previous_game) = hystory.pop() {
                    recovery.push(previous_game.clone());
                    if let reversi::Status::Running { current_turn: previous_player } = previous_game.get_status() {
                        if previous_player == current_turn {
                            game = previous_game;
                            interface::draw_board(&game);
                            continue 'turn;
                        }
                    }
                }

                while let Some(recovered_game) = recovery.pop() {
                    hystory.push(recovered_game.clone());
                }

                interface::no_undo_message(current_turn);
            }

            interface::UserCommand::Help => {
                println!("{}", interface::HELP);
                interface::draw_board(&game);
            }

            // Quit Match
            interface::UserCommand::Quit => {
                interface::quitting_message(current_turn);
                break;
            }

            _ => {
                panic!("Something's wrong here!");
            }
        }
    }
}
