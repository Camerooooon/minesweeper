use std::io;
use std::fmt::Display;
use crate::io::stdout;
use crate::io::stdin;
use termion::raw::IntoRawMode;
use std::io::Write;
use termion::input::TermRead;
use termion::event::Key;
use rand::*;

pub struct Minesweeper {
    board: Board,
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
                board.push_str(&format!(" {}", &self.cells[row * self.width + col].to_string()));
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
        if self.is_mine {
            return write!(f, "{}", "*");
        }
        return write!(f, "{}", self.adjacent_mines);
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
    io::stdin().read_line(&mut width).expect("Failed to read line");

    let mut height = String::new();
    println!("Enter the height of the board: ");
    io::stdin().read_line(&mut height).expect("Failed to read line");

    let mut mines = String::new();
    println!("Enter the number of mines: ");
    io::stdin().read_line(&mut mines).expect("Failed to read line");


    // Create the board
    let mut board = Board {
        width: width.trim().parse::<usize>().expect("Failed to parse width (did you provide a valid number)"),
        height: height.trim().parse::<usize>().expect("Failed to parse height (did you provide a valid number)"),
        mines: mines.trim().parse::<usize>().expect("Failed to parse mines (did you provide a valid number)"),
        cells: vec![],
        selected_row: 0,
        selected_col: 0,
    };
    board.cells = generate_cells(board.width, board.height);

    // Place the mines
    place_mines(&mut board.cells, board.mines);
    
    // Calculate all adjacent mines
    for index in 0..board.cells.len() {
        board.cells[index].adjacent_mines = adjacent_mines(&board, &board.cells[index]);
    }

    // Use termion to detect when movement keys are pressed
    
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Clear the screen and hide the cursor
    write!(stdout, "{}{}{}", termion::clear::All, termion::cursor::Hide, termion::cursor::Goto(1, 1)).unwrap();
    stdout.flush().unwrap();

    let mut game = Minesweeper {
        board: board,
    };

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
                let cell = &game.board.cells[cell_from_pos(game.board.selected_row as i8, game.board.selected_col as i8, &game.board).expect("Selected cell doesn't exist")];
                if cell.is_mine {
                    println!("You lost!");
                    break;
                }
            }
            Key::Char('\n') => {
            }
            _ => {},
        }
        render(&mut game);

    }

    // Reshow the cursor
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

pub fn adjacent_mines(board: &Board, cell: &Cell) -> i8 {
    let mut count = 0;

    let to_check = [
        relative_cell_index(-1, -1, cell, board),
        relative_cell_index(-1, 0, cell, board),
        relative_cell_index(-1, 1, cell, board),
        relative_cell_index(0, -1, cell, board),
        relative_cell_index(0, 1, cell, board),
        relative_cell_index(1, -1, cell, board),
        relative_cell_index(1, 0, cell, board),
        relative_cell_index(1, 1, cell, board),
    ];

    for index in to_check {
        match index {
            Some(i) => {
                if board.cells[i].is_mine {
                    count += 1;
                }
            },
            None => {}
        }
    }

    return count;
}

fn render(game: &Minesweeper) {
    let mut screen = "".to_string();
    screen += &format!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1)); 
    screen += &format!("{}\n", game.board);
    screen += &format!("r: {}, c: {}, enter: flag, space: safe", game.board.selected_row, game.board.selected_col);
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
