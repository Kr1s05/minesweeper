use std::collections::{HashSet, VecDeque};
use std::fmt::{Debug, Formatter};
use rand::Rng;
use crate::Cell::{Discovered, Pin};
use crate::Item::Empty;

struct Game {
    field: Vec<Vec<Cell>>,
    state: GameState,
    bombs: Vec<(usize, usize)>,
    pins: Vec<(usize, usize)>,
    hidden_empty: u32,
}
#[derive(Debug)]
enum GameState {
    Playing,
    Win,
    Loss,
}

#[derive(Clone)]
enum Item {
    Bomb,
    Empty(u8),
}
#[derive(Clone)]
enum Cell {
    Hidden(Item),
    Pin(Item),
    Discovered(Item),
}

impl Cell {
    fn get_item(&self) -> Item {
        match self {
            Cell::Hidden(item) | Pin(item) | Discovered(item) => item.clone()
        }
    }
    fn turn_discovered(&self) -> Cell {
        Discovered(self.get_item())
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut field_str = String::from("");
        for row in &self.field {
            for cell in row {
                match cell {
                    Cell::Hidden(_) => {
                        field_str.push_str(" _ ")
                    }
                    Pin(_) => {
                        field_str.push_str(" I ")
                    }
                    Discovered(item) => {
                        match item {
                            Item::Bomb => {
                                field_str.push_str(" H ")
                            }
                            Empty(i) => {
                                field_str.push_str(" ");
                                field_str.push_str(&*i.to_string());
                                field_str.push_str(" ")
                            }
                        }
                    }
                }
            }
            field_str.push_str("\n");
        }
        write!(f, "State: {:?}; Field:\n{}", self.state, field_str)
    }
}

impl Game {
    fn new(width: usize, height: usize) -> Game {
        let mut rng = rand::thread_rng();
        let bomb_budget = width * height * 15 / 100;
        let mut field = vec![vec![Cell::Hidden(Empty(0)); width]; height];
        let mut bombs: HashSet<(usize, usize)> = HashSet::new();
        while bombs.len() < bomb_budget {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            if bombs.insert((x, y)) {
                let row_start = if y > 0 { y - 1 } else { y };
                let row_end = (y + 1).min(height - 1);
                let col_start = if x > 0 { x - 1 } else { x };
                let col_end = (x + 1).min(width - 1);
                for i in row_start..row_end + 1 {
                    for j in col_start..col_end + 1 {
                        let c = &mut field[i][j];
                        if i == y && j == x {
                            *c = Cell::Hidden(Item::Bomb);
                        } else if let Cell::Hidden(Empty(count)) = c {
                            *count += 1;
                        }
                    }
                }
            }
        }
        let pins = Vec::new();
        Game {
            field,
            state: GameState::Playing,
            bombs: bombs.into_iter().collect(),
            pins,
            hidden_empty: (height*width - bomb_budget) as u32
        }
    }

    fn discover(&mut self, x: &usize, y: &usize) {
        let field = &mut self.field;
        match &field[*y][*x] {
            Cell::Hidden(item) => {
                println!("Discovering: {},{}", x, y);
                match item {
                    Item::Bomb => {
                        self.state = GameState::Loss;
                        for bomb in &self.bombs {
                            self.field[bomb.1][bomb.0] = Discovered(Item::Bomb)
                        }
                    }
                    Empty(score) => {
                        if *score > 0 {
                            field[*y][*x] = field[*y][*x].turn_discovered();
                            self.hidden_empty -= 1;
                            return;
                        }
                        fill_area(field, *y, *x, &mut self.hidden_empty);
                    }
                }
            }
            _ => return
        }
    }

    fn pin(&mut self, x: usize, y: usize) {
        let cell = &mut self.field[y][x];
        if let Cell::Hidden(_) = cell {
            *cell = Pin(cell.get_item());
            self.pins.push((x,y));
        }
    }

    fn update_game_state (&mut self) {
        if self.hidden_empty  == 0 {
            self.state = GameState::Win;
        }
    }
}

fn fill_area(field: &mut Vec<Vec<Cell>>, start_row: usize, start_col: usize, hidden_count: &mut u32) {
    let cols = field[0].len() as isize;
    let rows = field.len() as isize;
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)];
    let mut queue = VecDeque::new();
    queue.push_back((start_row, start_col));
    while let Some((row, col)) = queue.pop_front() {
        field[row][col] = field[row][col].turn_discovered();
        *hidden_count -= 1;
        if let Empty(count) = field[row][col].get_item() {
            if count > 0 {
                continue;
            }
            for &(r_offset, c_offset) in &directions {
                let row = row as isize + r_offset;
                let col = col as isize + c_offset;
                if row < 0 || row >= rows || col < 0 || col >= cols {
                    continue;
                }
                let (r, c) = (row as usize, col as usize);
                if let Cell::Hidden(Empty(_)) = field[r][c] {
                    queue.push_back((r, c));
                }
            }
        }
    }
}

fn main() {
    let mut game = Game::new(10, 5);
    println!("{:?}", &game);
    println!("{:?};{:?}", &game.bombs, &game.bombs.len());
    // game.discover(&8, &5);
    println!("{:?}", &game)
}
