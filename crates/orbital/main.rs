extern crate core;
extern crate system;

use std::cmp::{min, max};
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::io::{Read, Write, SeekFrom};
use std::mem;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use system::error::{Error, Result, EBADF};
use system::scheme::{Packet, Scheme};
use system::syscall::SYS_READ;

pub use self::color::Color;
pub use self::event::{Event, EventOption};
pub use self::font::Font;
pub use self::image::{Image, ImageRoi};
pub use self::socket::Socket;
pub use self::window::Window;

use self::bmp::BmpFile;
use self::event::{EVENT_KEY, EVENT_MOUSE, K_F1, QuitEvent};

pub mod bmp;
pub mod color;
#[path="../../kernel/common/event.rs"]
pub mod event;
pub mod font;
pub mod image;
pub mod socket;
pub mod window;

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect{
            x: x,
            y: y,
            w: w,
            h: h
        }
    }

    pub fn area(&self) -> i32 {
        self.w * self.h
    }

    pub fn container(&self, other: &Rect) -> Rect {
        let x = min(self.x, other.x);
        let y = min(self.y, other.y);

        Rect {
            x: x,
            y: y,
            w: max(self.x + self.w, other.x + other.w) - x,
            h: max(self.y + self.h, other.y + other.h) - y
        }
    }

    pub fn contains(&self, other: &Rect) -> bool {
        self.x <= other.x
        && self.x + self.w >= other.x + other.w
        && self.y <= other.y
        && self.y + self.h >= other.y + other.h
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        let x = max(self.x, other.x);
        let y = max(self.y, other.y);

        min(self.x + self.w, other.x + other.w) > x
        && min(self.y + self.h, other.y + other.h) > y
    }

    pub fn intersection(&self, other: &Rect) -> Rect {
        let x = max(self.x, other.x);
        let y = max(self.y, other.y);

        Rect {
            x: x,
            y: y,
            w: min(self.x + self.w, other.x + other.w) - x,
            h: min(self.y + self.h, other.y + other.h) - y
        }
    }
}

fn schedule(redraws: &mut Vec<Rect>, x: i32, y: i32, w: i32, h: i32) {
    //println!("schedule redraw: {},{} {},{}", x, y, w, h);

    let request = Rect::new(x, y, w, h);

    let mut push = true;
    for mut rect in redraws.iter_mut() {
        //If contained, ignore new redraw request
        if rect.contains(&request) {
            //println!("redraw ignored");
            push = false;
            break;
        } else {
            let container = rect.container(&request);
            if container.area() < rect.area() + request.area() {
                //println!("container more efficient");
                *rect = container;
                push = false;
                break;
            }
        }
    }

    if push {
        redraws.push(request);
    }
}

struct OrbitalScheme {
    start: Instant,
    image: Image,
    background: Image,
    cursor: Image,
    cursor_x: i32,
    cursor_y: i32,
    dragging: bool,
    drag_x: i32,
    drag_y: i32,
    next_id: isize,
    next_x: i32,
    next_y: i32,
    order: VecDeque<usize>,
    windows: BTreeMap<usize, Window>,
    redraws: Vec<Rect>,
    todo: Vec<Packet>
}

impl OrbitalScheme {
    fn new(width: i32, height: i32) -> OrbitalScheme {
        OrbitalScheme {
            start: Instant::now(),
            image: Image::new(width, height),
            background: BmpFile::from_path("/ui/background.bmp"),
            cursor: BmpFile::from_path("/ui/cursor.bmp"),
            cursor_x: 0,
            cursor_y: 0,
            dragging: false,
            drag_x: 0,
            drag_y: 0,
            next_id: 1,
            next_x: 20,
            next_y: 20,
            order: VecDeque::new(),
            windows: BTreeMap::new(),
            redraws: vec![Rect::new(0, 0, width, height)],
            todo: Vec::new()
        }
    }

    fn redraw(&mut self, display: &Socket){
        let mut redraws = Vec::new();
        mem::swap(&mut self.redraws, &mut redraws);

        //TODO: Optimize redraws

        for rect in redraws.iter() {
            //let elapsed = self.start.elapsed();
            //println!("redraw {} {}: {},{} {},{}", elapsed.secs, elapsed.nanos, rect.x, rect.y, rect.w, rect.h);

            self.image.roi(rect.x, rect.y, rect.w, rect.h)
                    .set(Color::rgb(75, 163, 253))
                    .blend(&self.background.roi(rect.x, rect.y, rect.w, rect.h));

            let mut i = self.order.len();
            for id in self.order.iter().rev() {
                i -= 1;
                if let Some(mut window) = self.windows.get_mut(&id) {
                    if rect.x < window.x + window.width() && rect.x + rect.w >= window.x && rect.y < window.y + window.height() && rect.y + rect.h >= window.y - 18 {
                        window.draw(&mut self.image, i == 0);
                    }
                }
            }

            if rect.x < self.cursor_x + self.cursor.width() && rect.x + rect.w >= self.cursor_x && rect.y < self.cursor_y + self.cursor.height() && rect.y + rect.h >= self.cursor_y {
                self.image.roi(self.cursor_x, self.cursor_y, self.cursor.width(), self.cursor.height()).blend(&self.cursor.as_roi());
            }

            let data = self.image.data();
            let x1 = max(0, min(self.image.width(), rect.x));
            let x2 = max(x1, min(self.image.width(), rect.x + rect.w));
            let y1 = max(0, min(self.image.height(), rect.y));
            let y2 = max(y1, min(self.image.height(), rect.y + rect.h));
            for row in y1..y2 {
                let off1 = row * self.image.width() + x1;
                let off2 = row * self.image.width() + x2;

                unsafe { display.seek(SeekFrom::Start(off1 as u64 * 4)).unwrap(); }
                display.send_type(&data[off1 as usize .. off2 as usize]).unwrap();
            }
        }
    }

    fn event(&mut self, event: Event){
        if event.code == EVENT_KEY {
            if event.b as u8 == K_F1 && event.c > 0 {
                ::std::process::Command::new("ps").spawn();
            }
            if let Some(id) = self.order.front() {
                if let Some(mut window) = self.windows.get_mut(&id) {
                    window.event(event);
                }
            }
        } else if event.code == EVENT_MOUSE {
            if event.a as i32 != self.cursor_x || event.b as i32 != self.cursor_y {
                schedule(&mut self.redraws, self.cursor_x, self.cursor_y, self.cursor.width(), self.cursor.height());
                self.cursor_x = event.a as i32;
                self.cursor_y = event.b as i32;
                schedule(&mut self.redraws, self.cursor_x, self.cursor_y, self.cursor.width(), self.cursor.height());
            }

            if self.dragging {
                if event.c > 0 {
                    if let Some(id) = self.order.front() {
                        if let Some(mut window) = self.windows.get_mut(&id) {
                            if self.drag_x != self.cursor_x || self.drag_y != self.cursor_y {
                                schedule(&mut self.redraws, window.x, window.y - 18, window.width(), window.height() + 18);
                                window.x += self.cursor_x - self.drag_x;
                                window.y += self.cursor_y - self.drag_y;
                                self.drag_x = self.cursor_x;
                                self.drag_y = self.cursor_y;
                                schedule(&mut self.redraws, window.x, window.y - 18, window.width(), window.height() + 18);
                            }
                        } else {
                            self.dragging = false;
                        }
                    } else {
                        self.dragging = false;
                    }
                } else {
                    self.dragging = false;
                }
            } else {
                let mut focus = 0;
                let mut i = 0;
                for id in self.order.iter() {
                    if let Some(mut window) = self.windows.get_mut(&id) {
                        if window.contains(event.a as i32, event.b as i32) {
                            let mut window_event = event;
                            window_event.a -= window.x as i64;
                            window_event.b -= window.y as i64;
                            window.event(window_event);
                            if event.c > 0 {
                                focus = i;
                            }
                            break;
                        } else if window.title_contains(event.a as i32, event.b as i32) {
                            if event.c > 0 {
                                focus = i;
                                if window.exit_contains(event.a as i32, event.b as i32) {
                                    window.event(QuitEvent.to_event());
                                } else {
                                    self.dragging = true;
                                    self.drag_x = self.cursor_x;
                                    self.drag_y = self.cursor_y;
                                }
                            }
                            break;
                        }
                    }
                    i += 1;
                }
                if focus > 0 {
                    if let Some(id) = self.order.remove(focus) {
                        self.order.push_front(id);
                    }
                }
            }
        }
    }
}

impl Scheme for OrbitalScheme {
    fn open(&mut self, path: &str, _flags: usize, _mode: usize) -> Result<usize> {
        let mut parts = path.split("/").skip(1);

        let mut x = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        let mut y = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        let width = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);
        let height = parts.next().unwrap_or("").parse::<i32>().unwrap_or(0);

        let mut title = parts.next().unwrap_or("").to_string();
        for part in parts {
            title.push('/');
            title.push_str(part);
        }

        let id = self.next_id as usize;
        self.next_id += 1;
        if self.next_id < 0 {
            self.next_id = 1;
        }

        if x < 0 && y < 0 {
            x = self.next_x;
            y = self.next_y;

            self.next_x += 20;
            if self.next_x + 20 >= self.image.width() {
                self.next_x = 20;
            }
            self.next_y += 20;
            if self.next_y + 20 >= self.image.height() {
                self.next_y = 20;
            }
        }

        self.order.push_front(id);
        self.windows.insert(id, Window::new(x, y, width, height, title));
        schedule(&mut self.redraws, x, y - 18, width, height + 18);

        Ok(id)
    }

    fn read(&mut self, id: usize, buf: &mut [u8]) -> Result<usize> {
        if let Some(mut window) = self.windows.get_mut(&id) {
            window.read(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn write(&mut self, id: usize, buf: &[u8]) -> Result<usize> {
        if let Some(mut window) = self.windows.get_mut(&id) {
            schedule(&mut self.redraws, window.x, window.y - 18, window.width(), window.height() + 18);
            window.write(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn fpath(&self, id: usize, buf: &mut [u8]) -> Result<usize> {
        if let Some(window) = self.windows.get(&id) {
            window.path(buf)
        } else {
            Err(Error::new(EBADF))
        }
    }

    fn close(&mut self, id: usize) -> Result<usize> {
        self.order.retain(|&e| e != id);

        if let Some(window) = self.windows.remove(&id) {
            schedule(&mut self.redraws, window.x, window.y - 18, window.width(), window.height() + 18);
            Ok(0)
        } else {
            Err(Error::new(EBADF))
        }
    }
}

fn event_loop(scheme_mutex: Arc<Mutex<OrbitalScheme>>, display: Arc<Socket>, socket: Arc<Socket>){
    loop {
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            scheme.redraw(&display);
        }

        let mut events = [Event::new(); 128];
        let count = display.receive_type(&mut events).unwrap();
        let mut responses = Vec::new();
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            for &event in events[.. count].iter() {
                scheme.event(event);
            }

            let mut packets = Vec::new();
            mem::swap(&mut scheme.todo, &mut packets);
            for mut packet in packets.iter_mut() {
                let delay = packet.a == SYS_READ;
                scheme.handle(packet);
                if delay && packet.a == 0 {
                    scheme.todo.push(*packet);
                }else{
                    responses.push(*packet);
                }
            }
        }
        if ! responses.is_empty() {
            socket.send_type(&responses).unwrap();
        }
    }
}

fn server_loop(scheme_mutex: Arc<Mutex<OrbitalScheme>>, display: Arc<Socket>, socket: Arc<Socket>){
    loop {
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            scheme.redraw(&display);
        }

        let mut packets = [Packet::default(); 128];
        let count = socket.receive_type(&mut packets).unwrap();
        let mut responses = Vec::new();
        {
            let mut scheme = scheme_mutex.lock().unwrap();
            for mut packet in packets[.. count].iter_mut() {
                let delay = packet.a == SYS_READ;
                scheme.handle(packet);
                if delay && packet.a == 0 {
                    scheme.todo.push(*packet);
                } else {
                    responses.push(*packet);
                }
            }
        }
        if ! responses.is_empty() {
            socket.send_type(&responses).unwrap();
        }
    }
}

fn main() {
    let socket = Socket::create(":orbital").map(|socket| Arc::new(socket)).unwrap();

    match Socket::open("display:").map(|display| Arc::new(display)) {
        Ok(display) => {
            let path = display.path().unwrap().to_string();
            let res = path.split(":").nth(1).unwrap_or("");
            let width = res.split("/").nth(0).unwrap_or("").parse::<i32>().unwrap_or(0);
            let height = res.split("/").nth(1).unwrap_or("").parse::<i32>().unwrap_or(0);

            println!("- Orbital: Found Display {}x{}", width, height);
            println!("    Console: Press F1");
            println!("    Desktop: Press F2");

            let scheme = Arc::new(Mutex::new(OrbitalScheme::new(width, height)));

            let scheme_event = scheme.clone();
            let display_event = display.clone();
            let socket_event = socket.clone();

            let server_thread = thread::spawn(move || {
                server_loop(scheme, display, socket);
            });

            event_loop(scheme_event, display_event, socket_event);

            server_thread.join().unwrap();
        },
        Err(err) => println!("- Orbital: No Display Found: {}", err)
    }
}
