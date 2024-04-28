use std::ops::Index;
use raylib::prelude::*;

struct Board<T> {
    board: Vec<T>,
    board2: Vec<T>,
    board1_active: bool,
    width: i32,
    height: i32,
}

impl<T> Board<T> {
    fn new(width: i32, height: i32) -> Board<T> where T : Clone + Default {
        Board::<T>{
            board: (0..width*height).map(|_| T::default()).collect(),
            board2: vec![T::default(); (width * height) as usize],
            board1_active: true,
            width: width,
            height: height
        }
    }

    fn write_at(&mut self, i :(i32, i32)) -> &mut T {
        let row = i.0.rem_euclid(self.height);
        let col = i.1.rem_euclid(self.width);
        if self.board1_active {
            &mut self.board2[(row * self.width + col) as usize]
        }
        else {
            &mut self.board[(row * self.width + col) as usize]
        }
    }

    fn step(&mut self, step_fn: fn(&Board<T>, &(i32, i32)) -> T) {
        for row in 0..self.height {
            for col in 0..self.width {
                *self.write_at((row, col)) = step_fn(&self, &(row, col))
            }
        }
        self.board1_active = !self.board1_active;
    }
}

impl<T> Index<(i32, i32)> for Board<T> {
    type Output = T;
    fn index(&self, i: (i32, i32)) -> &T {
        let row = i.0.rem_euclid(self.height);
        let col = i.1.rem_euclid(self.width);
        // TODO: use get unchecked
        if self.board1_active {
            &self.board[(row * self.width + col) as usize]
        }
        else {
            &self.board2[(row * self.width + col) as usize]
        }
    }
}


#[derive(Clone, Debug)]
enum GameOfLifeCell {
    Dead,
    Alive,
}

impl Default for GameOfLifeCell {
    fn default() -> Self {
        match rand::random() {
            true => GameOfLifeCell::Dead,
            false => GameOfLifeCell::Alive,
        }
    }
}

fn step(board: &Board<GameOfLifeCell>, (row, col): &(i32, i32)) -> GameOfLifeCell {
    const MOORE_NEIGHBOURHOOD: [(i32, i32); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1), (0, 1), // Not including self
        (1, -1), (1, 0), (1, 1)];
    let mut alive = 0;
    let center_cell = &board[(*row, *col)];
    for (delta_row, delta_col) in MOORE_NEIGHBOURHOOD {
        let cell: &GameOfLifeCell = &board[(row + delta_row, col +delta_col)];
        match cell {
            GameOfLifeCell::Alive => alive +=1,
            _ => (),
        }
    }
    match center_cell {
        GameOfLifeCell::Dead if alive == 3 => GameOfLifeCell::Alive,
        GameOfLifeCell::Dead => GameOfLifeCell::Dead,
        GameOfLifeCell::Alive if alive == 2 || alive == 3 => GameOfLifeCell::Alive,
        GameOfLifeCell::Alive => GameOfLifeCell::Dead,
    }
}

fn game_of_life(window_width: i32, window_height: i32, resizable: bool, cell_size: i32, ) {
    let (mut rl, thread) = raylib::init()
        .size(window_width as i32, window_height as i32)
        .title("Game Of Life")
        .build();

    WindowState::set_window_resizable(rl.get_window_state(), resizable);
    let mut automaton = Board::<GameOfLifeCell>::new(
        window_width / cell_size +1,
        window_height / cell_size +1, // TODO move this hack somewhere else
    );
    let mut single_step = true;
    while !rl.window_should_close() {
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            single_step = !single_step;
        }
        if single_step && rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            automaton.step(step);
        }
        if !single_step {
            automaton.step(step);
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLUE);
        for row in 0..automaton.height {
            for col in 0..automaton.width {
                let colour = match automaton[(row, col)] {
                    GameOfLifeCell::Alive => Color::WHITE,
                    GameOfLifeCell::Dead => Color::BLUE,
                };
                if colour != Color::BLUE {
                    let distance_center_to_middle_of_side: f32 = ((cell_size as f32).powi(2) - (cell_size as f32 / 2.0).powi(2)).sqrt();
                    let mut center_x = row as f32 * 2.0* distance_center_to_middle_of_side * (-60.0*DEG2RAD as f32).cos() + col as f32 * 2.0* distance_center_to_middle_of_side;
                    if center_x > window_width as f32 + cell_size as f32 {
                        center_x = row as f32 * 2.0* distance_center_to_middle_of_side * (-60.0*DEG2RAD as f32).cos() + (col - &automaton.width) as f32 * 2.0* distance_center_to_middle_of_side;
                    }

                    let center = Vector2::new(
                        center_x,
                        row as f32 * 2.0* distance_center_to_middle_of_side * (60.0*DEG2RAD as f32).sin()
                    );
                    //dbg!(center);
                    d.draw_poly(center , 6, cell_size as f32, 0.0, colour);
                }
                d.draw_fps(0, 0);
            }
        }
    }
}

const HELP: &str = "\
Hexagonal Automata v0.0.1

USAGE:
  app [FLAGS] [OPTIONS] <AUTOMATON>

FLAGS:
  -h, --help            Prints help information
  -r, --resizable       Use resizable window

OPTIONS:
  --width <width>       Set window width
  --height <height>     Set window height
  --cell_size <size>    Set cell size (center of hexagon to corner)

AUTOMATON:
  <AUTOMATON>           Cellular automaton to run (gol : Game of Life), 
";


fn main() {
    fn parse_number(s: &str) -> Result<i32, &'static str> {
        let n = s.parse().map_err(|_| "not a number");
        if n < Ok(1) {
            panic!("Invalid size");
        }
        n
    }
    let mut pargs = pico_args::Arguments::from_env();
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    let width : i32 = pargs.opt_value_from_fn("--width", parse_number).unwrap().unwrap_or(800);
    let height : i32 = pargs.opt_value_from_fn("--height", parse_number).unwrap().unwrap_or(600);
    let cell_size : i32 = pargs.opt_value_from_fn("--cell_size", parse_number).unwrap().unwrap_or(6);
    let resizable : bool = pargs.contains(["-r", "--resizable"]);
    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("WARNING: unknown arguments left: {:?}.", remaining);
    }
    game_of_life(width, height, resizable, cell_size);
    return;
    
}
