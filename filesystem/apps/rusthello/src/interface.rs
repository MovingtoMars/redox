// This module provides interface functionalities and manages all the input/output part of the program

use std::cmp::Ordering;
use std::io::{self, Write};
use players;
use reversi;

pub enum UserCommand {
	NewGame,
	NewPlayer(players::Player),
	Move(usize, usize),
	Help,
	Undo,
	Quit,
}

pub const INTRO: &'static str =
"\n\n
             RUSThello
             ● ○ ● ○ ●
       a simple Reversi game
     written in Rust with love
           Redox Edition
              v 1.1.0\n\n";

pub const MAIN_MENU: &'static str =
"\nMain Menu:
 n - New match
 h - Help
 q - Quit RUSThello";

 pub const NEW_PLAYER_MENU: &'static str =
 "\nChoose a player:
  hp - Human Player
  ai - Artificial Intelligence
  q  - Quit match";

pub const COMMANDS_INFO: &'static str =
"\nStarting new game…
Type a cell's coordinates to place your disk there. Exaple: \"c4\"
Type 'help' or 'h' instead of a move to display help message.
Type 'undo' or 'u' instead of a move to undo last move.
Type 'quit' or 'q' instead of a move to abandon the game.";

pub const HELP: &'static str = "\
\n\n\n\tHOW TO PLAY REVERSI:\n
Reversi is a board game where two players compete against each other. \
The game is played on a 8x8 board, just like chess but for the squares’ colour which is always green. \
There are 64 identical pieces called disks, which are white on one side and black on the other. \
A player is Light, using disks’ white side, and the other one is Dark, using disks' black side. \
The game starts with four disks already placed at the centre of the board, two for each side. \
Dark moves first.\n
Let’s say it’s Dark’s turn, for simplicity's sake, as for Light the rules are just the same. \
Dark has to place a disk in a free square of the board, with the black side facing up. \
Whenever the newly placed black disk and any other previously placed black disk enclose a sequence of white disks (horizontal, vertical or diagonal and of any length), all of those flip and turn black. \
It is mandatory to place the new disk such that at least a white disk is flipped, otherwise the move is not valid.\n
Usually players’ turn alternate, passing from one to the other. \
When a player cannot play any legal move, the turn goes back to the other player, thus allowing the same player to play consecutive turns. \
When neither player can play a legal move, the game ends. \
In particular, this is true whenever the board has been completely filled up (for a total of 60 moves), but games happen sometimes to end before that, leaving empty squares on the board.\n
When the game ends, the player with more disks turned to its side wins. \
Ties are possible as well, if both player have the same number of disks.\n\n\n
\tHOW TO USE RUSThello:\n
To play RUSThello you first have to choose who is playing on each side, Dark and Light. \
You can choose a human players or an AI. \
Choose human for both players and challenge a friend, or test your skills against an AI, or even relax and watch as two AIs compete with each other: all matches are possible!\n
As a human player, you move by entering the coordinates (a letter and a number) of the square you want to place your disk on, e.g. all of 'c4', 'C4', '4c' and '4C' are valid and equivalent coordinates. \
For your ease of use, all legal moves are marked on the board with a *.\n
Furthermore, on your turn you can also input special commands: 'undo' to undo your last move (and yes, you can 'undo' as many times as you like) and 'quit' to quit the game.\n\n\n
\tCREDITS:\n
RUSThello v. 1.1.0 Redox Edition
by Enrico Ghiorzi, with the invaluable help of the Redox community
Copyright (c) 2015 by Enrico Ghiorzi
Released under the MIT license\n\n\n";


pub fn input_main_menu() -> UserCommand {

	loop {
		print!("Insert input: ");
		match get_user_command() {
			Some(UserCommand::NewGame)	=> return UserCommand::NewGame,
			Some(UserCommand::Help) 	=> return UserCommand::Help,
			Some(UserCommand::Quit) 	=> {
				println!("\nGoodbye!\n\n\n");
				return UserCommand::Quit;
			}
			_ => println!("This is not a valid command!"),
		}
	}
}

pub fn new_player(side: reversi::Disk) -> Option<players::Player> {
	loop {
		match side {
			reversi::Disk::Light => print!("● Light player: "),
			reversi::Disk::Dark  => print!("○ Dark  player: "),
		}
		match get_user_command() {
			Some(UserCommand::NewPlayer(player)) => return Some(player),
			Some(UserCommand::Quit) => return None,
			_ => println!("This is not a valid command!"),
		}
	}
}

/// It gets an input from the user and tries to parse it, then returns a Option<UserCommand>`.
/// If the input is recognized as a legit command, it returns the relative `Option::Some(UserCommand)`.
/// If the input is not recognized as a legit command, it returns a `Option::None`.
pub fn get_user_command() -> Option<UserCommand> {

    // Read the input
    let _ = io::stdout().flush();

    let mut input = String::new();

    io::stdin().read_line(&mut input)
        .ok()
        .expect("failed to read line");

	let input = input.trim().to_lowercase();

	match &*input {
		"hp" => Some(UserCommand::NewPlayer(players::Player::Human)),
		"ai" => Some(UserCommand::NewPlayer(players::Player::AiMedium)),
		"n" | "new game"	=> Some(UserCommand::NewGame),
		"h" | "help" 		=> Some(UserCommand::Help),
		"u" | "undo" 		=> Some(UserCommand::Undo),
		"q" | "quit" 		=> Some(UserCommand::Quit),
		_	=> {

			let mut row: Option<usize> = None;
			let mut col: Option<usize> = None;

			for curr_char in input.chars() {
				match curr_char {
					'1'...'8'	=> {
						if let None = row {
							row = Some(curr_char as usize - '1' as usize);
						} else {
							return None;
						}
					}
					'a'...'h'	=> {
						if let None = col {
							col = Some(curr_char  as usize - 'a' as usize);
						} else {
							return None;
						}
					}
					_			=> return None,
				}
			}

			if row.is_none() || col.is_none() {
				None
			} else {
				// The move is not checked!
				Some(UserCommand::Move(row.unwrap(), col.unwrap()))
			}
		}
	}
}



/// draw_board draws the board (using text characters) in a pleasant-looking way, converting the board in a string (board_to_string) and then printing this.
pub fn draw_board(game: &reversi::Game) {

    let board = game.get_board();

    // Declare board_to_string and add column reference at the top
    let mut board_to_string: String = "\n\n\n\t   a  b  c  d  e  f  g  h\n".to_string();

    // For every row add a row reference to the left
    for (row, row_array) in board.iter().enumerate() {
        board_to_string.push('\t');
        board_to_string.push_str(&(row + 1).to_string());
        board_to_string.push(' ');

        // For every column, add the appropriate character depending on the content of the current cell
        for (col, &cell) in row_array.iter().enumerate() {

            match cell {
                // Light and Dark cells are represented by white and black bullets
                reversi::Cell::Taken { disk: reversi::Disk::Light } => board_to_string.push_str(" ● "),
                reversi::Cell::Taken { disk: reversi::Disk::Dark }  => board_to_string.push_str(" ○ "),

                // An empty cell will display a plus or a multiplication sign if the current player can move in that cell
                // or a little central dot otherwise
                reversi::Cell::Empty => {
                    if game.check_move((row, col)) {
                        if let reversi::Status::Running { .. } = game.get_status() {
							board_to_string.push_str(" * ");
                        }
                    } else {
                        board_to_string.push_str(" ∙ ");
                    }
                }
            }
        }

        // Add a row reference to the right
        board_to_string.push(' ');
        board_to_string.push_str(&(row + 1).to_string());
        board_to_string.push('\n');
    }

    // Add column reference at the bottom
    board_to_string.push_str("\t   a  b  c  d  e  f  g  h\n");

    // Print board
    println!("{}", board_to_string);

    // Print current score and game info
    let (score_light, score_dark) = game.get_score();

    match game.get_status() {
        reversi::Status::Running { current_turn } => {
            match current_turn {
                reversi::Disk::Light => println!("\t        {:>2} ○ >> ● {:<2}\n", score_dark, score_light),
                reversi::Disk::Dark  => println!("\t        {:>2} ○ << ● {:<2}\n", score_dark, score_light),
            }
        }
        reversi::Status::Ended => {
            println!("\t        {:>2} ○    ● {:<2}\n", score_dark, score_light);
            match score_light.cmp(&score_dark) {
                Ordering::Greater => println!("Light wins!"),
                Ordering::Less    => println!("Dark wins!"),
                Ordering::Equal   => println!("Draw!"),
            }
        }
    }
}



/// Prints a message with info on a move.
pub fn print_move(game: &reversi::Game, (row, col): (usize, usize)) {

    let char_col = (('a' as u8) + (col as u8)) as char;
    if let reversi::Status::Running { current_turn } = game.get_status() {
        match current_turn {
            reversi::Disk::Light => println!("● Light moves: {}{}", char_col, row + 1),
            reversi::Disk::Dark  => println!("○ Dark moves: {}{}",  char_col, row + 1),
        }
    }
}



/// It get_status a human player's input and convert it into a move.
/// If the move if illegal, it ask for another input until the given move is a legal one.
pub fn human_make_move(game: &reversi::Game) -> UserCommand {

    if let reversi::Status::Running { current_turn } = game.get_status() {
        match current_turn {
            reversi::Disk::Light => print!("● Light moves: "),
            reversi::Disk::Dark  => print!("○ Dark moves: "),
        }
    }

    loop {
		if let Some(user_command) = get_user_command() {
			match user_command {
				UserCommand::Move(row, col) => {
					if game.check_move((row, col)) {
						return UserCommand::Move(row, col);
					} else {
						print!("Illegal move, try again: ");
						continue;
					}
				}
				_ => return user_command,
			}
		} else {
			print!("This doesn't look like a valid command. Try again: ");
			continue;
		}
    }
}



// Print a last message before a player quits the game
pub fn quitting_message(coward: reversi::Disk) {
    match coward {
        reversi::Disk::Light => println!("Light is running away, the coward!"),
        reversi::Disk::Dark  => println!("Dark is running away, the coward!"),
    }
}

// Print a last message when 'undo' is not possible
pub fn no_undo_message(undecided: reversi::Disk) {
	match undecided {
        reversi::Disk::Light => println!("There is no move Light can undo."),
        reversi::Disk::Dark  => println!("There is no move Dark can undo."),
    }
}
