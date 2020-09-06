extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

//use std::process;
use piston::window::WindowSettings;
use piston::event_loop::{EventSettings, Events};
use piston::input::{Button, Key, PressEvent, ReleaseEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};


use rand::{
    distributions::{Distribution, Standard},
    Rng,
}; // 0.7.0


const BOARDSIZE_X : i32 = 10;
const BOARDSIZE_Y : i32 = 20;



#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Color {
    Red = 0, Green, Blue, Magenta, Cyan, Yellow, Orange,
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0, 7) {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Magenta,
            4 => Color::Cyan,
            5 => Color::Yellow,
            _ => Color::Orange,
		}
    }
}

fn get_color_index(col : Color) -> i32
{
	return (col as i32) + 1;
}

 pub struct Block
 {
	blocks: Vec<i32>,
	size_x: i32,
	size_y: i32,
 }

pub struct App {
    gl: GlGraphics,
    score: i32,
    block_x: i32,
	block_y: i32,
	block_rotate: i32,
    block_type: Color,
    drop_timer: u128,
	last_drop: std::time::Instant,
	
    possible_pieces: Vec<Block>,
    arr: [i32; (BOARDSIZE_X * (BOARDSIZE_Y + 4)) as usize]
}

#[derive(Clone,Copy,Debug)]
pub struct ColorBlock
{
	c: [f32; 4]
}

static COLORS: [ColorBlock; 7] =  [
	ColorBlock{c: [1.0, 0.0, 0.0, 1.0]},
	ColorBlock{c: [0.0, 1.0, 0.0, 1.0]},
	ColorBlock{c: [0.0, 0.0, 1.0, 1.0]},
	ColorBlock{c: [1.0, 0.0, 1.0, 1.0]},
	ColorBlock{c: [0.0, 1.0, 1.0, 1.0]},
	ColorBlock{c: [1.0, 1.0, 0.0, 1.0]},
	ColorBlock{c: [1.0, 0.65, 0.0, 1.0]},
	];


fn get_x_pos(block : &Block, block_x : i32, block_rotate : i32, x : i32, y : i32) -> i32
{
	if block_rotate == 0
	{
		return block_x + x - block.size_x / 2;
	}
	if block_rotate == 2
	{
		return block_x - x + block.size_x / 2;
	}
	if block_rotate == 3
	{
		return block_x + y - block.size_y / 2;
	}
	return block_x - y + block.size_y / 2;
}

fn get_y_pos(block : &Block, block_y : i32, block_rotate : i32, x : i32, y : i32) -> i32
{
	if block_rotate == 0
	{
		return block_y + y - block.size_y / 2;
	}
	if block_rotate == 2
	{
		return block_y - y + block.size_y / 2;
	}
	if block_rotate == 1
	{
		return block_y + x - block.size_x / 2;
	}
	return block_y - x + block.size_x / 2;
}


impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BACKGROUND: [f32; 4] = [0.0, 0.5, 0.5, 1.0];

		let ball = rectangle::square(0.0, 0.0, 10.0);

		let arr = &self.arr;
		
		let block = &self.possible_pieces[self.block_type as usize];
		let block_x = self.block_x;
		let block_y = self.block_y;
		let block_rotate = self.block_rotate;
		let col = (get_color_index(self.block_type) - 1) as usize;
		
        self.gl.draw(args.viewport(), |c, gl| {
            clear(BACKGROUND, gl);
            
            for ind in 0..200
            {
                let val = arr[ind];
                let x = (ind % (BOARDSIZE_X) as usize) as f64;
                let y = ((ind / (BOARDSIZE_X) as usize)) as f64;
                if val != 0
                {
					let tmp_y = (BOARDSIZE_Y - 1) as f64 - y;
                    rectangle(COLORS[(val - 1) as usize].c, ball, c.transform.trans(x * 10.0, tmp_y * 10.0), gl);        
                }
            }

			
			for y in 0 .. block.size_y
			{
				for x in 0 .. block.size_x
				{
					let xpos = get_x_pos(block, block_x, block_rotate, x, y) as f64;
					let ypos = get_y_pos(block, block_y, block_rotate, x, y) as f64;
					if block.blocks[(x + y * block.size_x) as usize] != 0
					{
						let tmp_y = (BOARDSIZE_Y - 1)as f64 - ypos;
						rectangle(COLORS[col].c, ball, c.transform.trans(xpos * 10.0, tmp_y * 10.0), gl);        
					}

				}	
			}
			
            
        });
    }

	fn check_hit(&mut self, dir_x: i32, dir_y: i32, check_top : bool) -> i32
	{
		let block = &self.possible_pieces[self.block_type as usize];
		let mut hits: i32 = 0;
		let block_x = self.block_x;
		let block_y = self.block_y;
		let block_rotate = self.block_rotate;
		//if !hits
		{
			for y in 0 .. block.size_y
			{
				for x in 0 .. block.size_x
				{

					let xpos = get_x_pos(block, block_x, block_rotate, x, y) + dir_x;
					let ypos = get_y_pos(block, block_y, block_rotate, x, y) + dir_y;

					if xpos < 0
					{
						hits |= 2;
					}
					else if xpos >= BOARDSIZE_X
					{
						hits |= 4;
					}
					
					else if ypos >= BOARDSIZE_Y && check_top
					{
						hits |= 8;
						return hits;
					}
					
					else if ypos < 0
					{
						hits = 1;
					}
					else if (block.blocks[(x + y * block.size_x) as usize] != 0 ) && (self.arr[(xpos + ypos * BOARDSIZE_X) as usize] != 0)
					{
						hits = 1;
					}
					
				}
			}
		}
		return hits;
	}

	fn remove_row(&mut self, row_number : i32) -> bool
	{
		let arr = &mut self.arr;

		let mut remove : bool = true;
		for x in 0 .. BOARDSIZE_X
		{
			remove &= arr[(x + row_number * BOARDSIZE_X) as usize] != 0;
		}
		
		if remove
		{


			if row_number < BOARDSIZE_Y - 1
			{
				for y in row_number .. BOARDSIZE_Y
				{
					for x in 0 .. BOARDSIZE_X
					{
						arr[(x + y * BOARDSIZE_X) as usize] = arr[(x + (y + 1) * BOARDSIZE_X) as usize];
					}
				}
			}
			
			// replace rownumber 20
			{
				for x in 0 .. BOARDSIZE_X
				{
					arr[(x + (BOARDSIZE_Y - 1) * BOARDSIZE_X) as usize] = 0;
				}
			}
		}

		return remove;
	}

	fn row_down(&mut self)
	{

		let hits = self.check_hit(0, -1, false);
		let block = &self.possible_pieces[self.block_type as usize];
		let block_x = self.block_x;
		let block_y = self.block_y;
		let block_rotate = self.block_rotate;
		if hits != 0
		{
			let col = get_color_index(self.block_type);
			for y in 0 .. block.size_y
			{
				for x in 0 .. block.size_x
				{
					let xpos = get_x_pos(block, block_x, block_rotate, x, y);
					let ypos = get_y_pos(block, block_y, block_rotate, x, y);
					if ypos >= 0 && block.blocks[(x + y * block.size_x) as usize] != 0
					{
						self.arr[(xpos + ypos * BOARDSIZE_X) as usize] = col;
					}
				}
	
			}
			// Game over
			if (self.check_hit(0, 0, true) & 8) == 8
			{
				println!("Game over!");
				for ind in 0 .. BOARDSIZE_X * (BOARDSIZE_Y + 4)
				{
					self.arr[ind as usize] = 0;
				}
			}

			self.block_type = rand::random();
			self.block_x = 4;
			self.block_y = 20;
			self.block_rotate = 0;
			for y in 0 .. BOARDSIZE_Y
			{
				while self.remove_row(y){}

			}
		}
		else
		{
			self.block_y = self.block_y - 1;
		}
		self.last_drop = std::time::Instant::now();
	}

    fn update(&mut self, _args: &UpdateArgs) 
    {
		if self.last_drop.elapsed().as_millis() > self.drop_timer
		{
			self.row_down();
		}
    }

	fn rotate(&mut self)
	{
		self.block_rotate = (self.block_rotate + 1) % 4;
		let mut check_status = self.check_hit(0, 0, false);
		while check_status != 0
		{
			if check_status == 2
			{
				self.block_x = self.block_x + 1;
			}
			else if check_status == 4
			{
				self.block_x = self.block_x - 1;
			}
			else if (check_status & 8) != 8
			{
				self.block_y = self.block_y + 1;
			}
			check_status = self.check_hit(0, 0, false);
		}
	}

    fn press(&mut self, args: &Button) {
		
		if let &Button::Keyboard(key) = args {

            match key {
                Key::Up => {
					self.rotate();
                }
                Key::Down => {
					self.row_down();
                }
                Key::Left => {
					if self.check_hit(-1, 0, false) == 0
					{

						self.block_x = self.block_x - 1;
					}
                }
                Key::Right => {
					if self.check_hit(1, 0, false) == 0
					{

						self.block_x = self.block_x + 1;
					}
                }
                
                Key::W => {
					self.rotate();
                }
                Key::S => {
					self.row_down();
                }
                Key::A => {
					if self.check_hit(-1, 0, false) == 0
					{
						self.block_x = self.block_x - 1;
					}
                }
                Key::D => {
					if self.check_hit(1, 0, false) == 0
					{
						self.block_x = self.block_x + 1;
					}
                }
                _ => {}
			}
        }
    }

    fn release(&mut self, args: &Button) {
        if let &Button::Keyboard(key) = args {
            match key {
                Key::Up => {
                }
                Key::Down => {
                }
                Key::W => {
                }
                Key::S => {
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let opengl = OpenGL::V3_2;
    let mut window: GlutinWindow = WindowSettings::new("Pong", [512, 342])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        score: 0,
        block_x: 4,
		block_y: 19,
		block_rotate: 0,
        block_type: Color::Red,
        drop_timer: 100,
		last_drop: std::time::Instant::now(),
        possible_pieces: vec![
			Block{ blocks: vec![1, 1, 1, 1],  size_x: 2, size_y: 2 },
			Block{ blocks: vec![1, 1, 1, 0, 0, 1], size_x: 3, size_y: 2 },
			Block{ blocks: vec![1, 1, 1, 1], size_x: 4, size_y: 1 },
			Block{ blocks: vec![1, 1, 0, 0, 1, 1], size_x: 3, size_y: 2 },
			Block{ blocks: vec![0, 1, 1, 1, 1, 0], size_x: 3, size_y: 2 },
			Block{ blocks: vec![0, 1, 0, 1, 1, 1], size_x: 3, size_y: 2 },
			Block{ blocks: vec![1, 1, 1, 1, 0, 0], size_x: 3, size_y: 2 },
        ],
        arr: [0; (BOARDSIZE_X * (BOARDSIZE_Y + 4)) as usize],
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(b) = e.press_args() {
            app.press(&b);
        }

        if let Some(b) = e.release_args() {
            app.release(&b);
        }
    }
}