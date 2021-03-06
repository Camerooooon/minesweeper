use crate::io::stdin;
use crate::io::stdout;
use rand::*;
use std::fmt::Display;
use std::io;
use std::io::Write;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Minesweeper {
    board: Board,
    first_move: bool,
}

pub struct Board {
    width: usize,
    height: usize,
    mines: usize,
    cells: Vec<Cell>,
    selected_row: usize,
    selected_col: usize,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut board = String::new();
        for row in 0..self.height {
            for col in 0..self.width {
                if row == self.selected_row && col == self.selected_col {
                    board.push_str(&format!("{}", termion::style::Bold));
                }
                board.push_str(&format!(
                    " {}",
                    &self.cells[row * self.width + col].to_string()
                ));
                if row == self.selected_row && col == self.selected_col {
                    board.push_str(&format!("{}", termion::style::Reset));
                }
            }
            board.push_str("\r\n");
        }
        // Remove the final newline
        board.pop();
        return write!(f, "{}", board);
    }
}

pub struct Cell {
    is_mine: bool,
    is_revealed: bool,
    is_flagged: bool,
    adjacent_mines: i8,
    row: usize,
    col: usize,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_revealed {
            if self.is_mine {
                return write!(f, "{}", "*");
            }
            return write!(f, "{}", self.adjacent_mines);
        } else if self.is_flagged {
            return write!(f, "{}", "F");
        }
        return write!(f, "{}", "\u{2022}");
    }
}

pub fn generate_cells(width: usize, height: usize) -> Vec<Cell> {
    let mut cells = Vec::new();
    for row in 0..height {
        for col in 0..width {
            cells.push(Cell {
                is_mine: false,
                is_revealed: false,
                is_flagged: false,
                adjacent_mines: 0,
                row: row,
                col: col,
            });
        }
    }
    return cells;
}

pub fn place_mines(cells: &mut Vec<Cell>, mines: usize) {
    let mut rng = rand::thread_rng();
    let mut mines_placed = 0;
    while mines_placed < mines {
        let index = rng.gen_range(0..cells.len());
        if !cells[index].is_mine {
            cells[index].is_mine = true;
            mines_placed += 1;
        }
    }
}

fn main() {
    // Get the board size from the user
    let mut width = String::new();
    println!("Enter the width of the board: ");
    io::stdin()
        .read_line(&mut width)
        .expect("Failed to read line");

    let mut height = String::new();
    println!("Enter the height of the board: ");
    io::stdin()
        .read_line(&mut height)
        .expect("Failed to read line");

    let mut mines = String::new();
    println!("Enter the number of mines: ");
    io::stdin()
        .read_line(&mut mines)
        .expect("Failed to read line");

    // Create the board
    let mut board = Board {
        width: width
            .trim()
            .parse::<usize>()
            .expect("Failed to parse width (did you provide a valid number)"),
        height: height
            .trim()
            .parse::<usize>()
            .expect("Failed to parse height (did you provide a valid number)"),
        mines: mines
            .trim()
            .parse::<usize>()
            .expect("Failed to parse mines (did you provide a valid number)"),
        cells: vec![],
        selected_row: 0,
        selected_col: 0,
    };
    board.cells = generate_cells(board.width, board.height);

    // Place the mines
    place_mines(&mut board.cells, board.mines);

    // Calculate all adjacent mines
    calculate_adjacent_mines(&mut board);

    // Use termion to detect when movement keys are pressed

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Clear the screen and hide the cursor
    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Hide,
        termion::cursor::Goto(1, 1)
    )
    .unwrap();
    stdout.flush().unwrap();

    let mut game = Minesweeper { board: board, first_move: true };

    render(&mut game);
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Ctrl('c') | Key::Char('q') => break,
            Key::Left => {
                if game.board.selected_col > 0 {
                    game.board.selected_col -= 1;
                }
            }
            Key::Right => {
                if game.board.selected_col < game.board.width - 1 {
                    game.board.selected_col += 1;
                }
            }
            Key::Up => {
                if game.board.selected_row > 0 {
                    game.board.selected_row -= 1;
                }
            }
            Key::Down => {
                if game.board.selected_row < game.board.height - 1 {
                    game.board.selected_row += 1;
                }
            }
            Key::Char(' ') => {
                let cell_index = cell_from_pos(
                    game.board.selected_row as i8,
                    game.board.selected_col as i8,
                    &game.board,
                )
                .expect("Selected cell doesn't exist");
                if game.board.cells[cell_index].is_mine {
                    if game.first_move {
                        game.board.cells[cell_index].is_mine = false;
                        calculate_adjacent_mines(&mut game.board);
                    } else {
                        println!("You lost!");
                        break;
                    }
                }
                let cell = &mut game.board.cells[cell_index];
                if !cell.is_revealed {
                    if cell.adjacent_mines == 0 {
                        reveal_cells_around(&mut game.board, cell_index);
                    }
                    game.board.cells[cell_index].is_revealed = true;
                }
                if game.first_move {
                    game.first_move = false;
                }
            }
            Key::Char('f') => {
                let cell_index = cell_from_pos(
                    game.board.selected_row as i8,
                    game.board.selected_col as i8,
                    &game.board,
                )
                .expect("Selected cell doesn't exist");
                game.board.cells[cell_index].is_flagged = !game.board.cells[cell_index].is_flagged;
            }
            Key::Char('\n') => {}
            _ => {}
        }
        render(&mut game);
    }

    // Reshow the cursor
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

pub fn calculate_adjacent_mines(board: &mut Board) {
    for index in 0..board.cells.len() {
        board.cells[index].adjacent_mines = adjacent_mines(&board, &board.cells[index]);
    }
}

pub fn reveal_cells_around(board: &mut Board, cell_index: usize) {
    let cell = &board.cells[cell_index];
    for cell_near_index_opt in cells_around(&board, &cell) {
        match cell_near_index_opt {
            Some(cell_near_index) => {
                if board.cells[cell_near_index].is_mine == false
                    && board.cells[cell_near_index].is_revealed == false
                {
                    board.cells[cell_near_index].is_revealed = true;
                    if board.cells[cell_near_index].adjacent_mines == 0 {
                        reveal_cells_around(board, cell_near_index);
                    }
                }
            }
            None => {}
        }
    }
}

pub fn cells_around(board: &Board, cell: &Cell) -> Vec<Option<usize>> {
    return vec![
        relative_cell_index(-1, -1, cell, board),
        relative_cell_index(-1, 0, cell, board),
        relative_cell_index(-1, 1, cell, board),
        relative_cell_index(0, -1, cell, board),
        relative_cell_index(0, 1, cell, board),
        relative_cell_index(1, -1, cell, board),
        relative_cell_index(1, 0, cell, board),
        relative_cell_index(1, 1, cell, board),
    ];
}

pub fn adjacent_mines(board: &Board, cell: &Cell) -> i8 {
    let mut count = 0;
    for index in cells_around(&board, &cell) {
        match index {
            Some(i) => {
                if board.cells[i].is_mine {
                    count += 1;
                }
            }
            None => {}
        }
    }

    return count;
}

fn render(game: &Minesweeper) {
    let mut screen = "".to_string();
    screen += &format!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
    screen += &format!("{}\n", game.board);
    screen += &format!(
        "r: {}, c: {}, enter: flag, space: safe",
        game.board.selected_row, game.board.selected_col
    );
    // Draw stdout from top left relative
    println!("{}", screen);
}

fn relative_cell_index(delta_row: i8, delta_col: i8, cell: &Cell, board: &Board) -> Option<usize> {
    let row = cell.row as i8 + delta_row;
    let col = cell.col as i8 + delta_col;
    if row < 0 || row >= board.height as i8 || col < 0 || col >= board.width as i8 {
        return None;
    }
    for (index, iter_cell) in board.cells.iter().enumerate() {
        if iter_cell.row as i8 == row && iter_cell.col as i8 == col {
            return Some(index);
        }
    }
    None
}

fn cell_from_pos(row: i8, col: i8, board: &Board) -> Option<usize> {
    for (index, iter_cell) in board.cells.iter().enumerate() {
        if iter_cell.row as i8 == row && iter_cell.col as i8 == col {
            return Some(index);
        }
    }
    None
}
