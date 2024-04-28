use std::ops::Index;

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

#[link(name = "raylib")]
extern "C" {
    // void InitWindow(int width, int height, const char *title);
    fn InitWindow(width: std::ffi::c_int, height: std::ffi::c_int, title: *const std::ffi::c_char);
    // void SetWindowState(unsigned int flags);
    fn SetWindowState(flags: std::ffi::c_uint);
    // bool WindowShouldClose(void);
    fn WindowShouldClose() -> bool;
    // bool IsKeyPressed(int key);
    fn IsKeyPressed(key: i32) -> bool;
    // void BeginDrawing(void);
    fn BeginDrawing();
    // void EndDrawing(void);
    fn EndDrawing();
    // void ClearBackground(Color color); 
    fn ClearBackground(color: Color);
    // void DrawPoly(Vector2 center, int sides, float radius, float rotation, Color color);
    fn DrawPoly(center: Vector2, sides: std::ffi::c_int, radius: std::ffi::c_float, rotation: std::ffi::c_float, color: Color);
    // void CloseWindow(void);
    fn CloseWindow();

}
const FLAG_WINDOW_RESIZABLE : std::ffi::c_uint = 0x00000004;
const KEY_S : std::ffi::c_int = 83;
const KEY_ENTER: std::ffi::c_int = 257;
const BLUE: Color = Color{r: 0, g:121, b:241, a: 255 };
const WHITE: Color = Color{r: 255, g: 255, b: 255, a: 255 };
const DEG2RAD: f32 = std::f32::consts::PI / 180.0;
#[repr(C)]
#[derive(PartialEq, Eq)]
struct Color {
    r: std::ffi::c_uchar,
    g: std::ffi::c_uchar,
    b: std::ffi::c_uchar,
    a: std::ffi::c_uchar,
}

#[repr(C)]
struct Vector2 {
    x: std::ffi::c_float,
    y: std::ffi::c_float,
}

fn game_of_life(window_width: i32, window_height: i32, _resizable: bool, cell_size: i32, ) {
    unsafe {InitWindow(window_width, window_height, "Hello".as_ptr() as *const std::ffi::c_char)};
    unsafe {SetWindowState(FLAG_WINDOW_RESIZABLE); };
    let mut automaton = Board::<GameOfLifeCell>::new(
        window_width / cell_size +1,
        window_height / cell_size +1, // TODO move this hack somewhere else
    );
    let mut single_step = true;
    while unsafe{WindowShouldClose() == false} {
        if unsafe {IsKeyPressed(KEY_S)} {
            single_step = !single_step;
        }
        if single_step && unsafe {IsKeyPressed(KEY_ENTER) } {
            automaton.step(step);
        }
        if !single_step {
            automaton.step(step);
        }
        unsafe {BeginDrawing()};
        unsafe {ClearBackground(BLUE)};
        for row in 0..automaton.height {
            for col in 0..automaton.width {
                let colour = match automaton[(row, col)] {
                    GameOfLifeCell::Alive => WHITE,
                    GameOfLifeCell::Dead => BLUE,
                };
                if colour != BLUE {
                    let distance_center_to_middle_of_side: f32 = ((cell_size as f32).powi(2) - (cell_size as f32 / 2.0).powi(2)).sqrt();
                    let mut center_x = row as f32 * 2.0* distance_center_to_middle_of_side * (-60.0*DEG2RAD as f32).cos() + col as f32 * 2.0* distance_center_to_middle_of_side;
                    if center_x > window_width as f32 + cell_size as f32 {
                        center_x = row as f32 * 2.0* distance_center_to_middle_of_side * (-60.0*DEG2RAD as f32).cos() + (col - &automaton.width) as f32 * 2.0* distance_center_to_middle_of_side;
                    }

                    let center = Vector2{
                        x: center_x,
                        y: row as f32 * 2.0* distance_center_to_middle_of_side * (60.0*DEG2RAD as f32).sin()
                    };
                    unsafe {DrawPoly(center , 6, cell_size as f32, 30.0, WHITE) };
                }
            }
        }
        unsafe{EndDrawing()};
    }
    unsafe {CloseWindow()};
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
