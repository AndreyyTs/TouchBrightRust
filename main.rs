use std::fs::OpenOptions;
use std::io::{self, ErrorKind, Read, Write};
use std::mem;
use std::os::unix::fs::OpenOptionsExt;
use std::fs::File;


const TOUCHPAD_DEVPATH: &str = "/dev/input/event5";
const BRIGHTNESS_PATH: &str = "/sys/class/backlight/intel_backlight/brightness";
const O_RDONLY: i32 = 0;

const EV_ABS: u16 = 0x03;
const ABS_X: u16 = 0x00;
const ABS_Y: u16 = 0x01;

const EV_KEY: u16 = 0x01;
const BTN_TOUCH: u16 = 0x14A;

#[repr(C)]
#[derive(Debug)]
struct TimeVal {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
#[derive(Debug)]
struct InputEvent {
    time: TimeVal,
    type_: u16,
    code: u16,
    value: i32,
}

fn main() -> io::Result<()> {
    let mut touchpad_pressed: bool = false;  
    let mut position_x: i32 = -1;  
    let mut position_y: i32 = -1;  

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(BRIGHTNESS_PATH)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let mut brightness: u64 = match contents.trim().parse() {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Err");
            return Ok(());
        }
    };
    println!("brightness: {}", brightness);

    let mut touchpad = OpenOptions::new()
        .read(true)
        .custom_flags(O_RDONLY)
        .open(TOUCHPAD_DEVPATH)?;

    let mut buf = [0u8; mem::size_of::<InputEvent>()];

    loop {
        match touchpad.read(&mut buf) {
            Ok(n) if n == buf.len() => {
                let event: InputEvent = unsafe { mem::transmute_copy(&buf) };
                //println!("{:?}", event);

                // косание
                if (event.type_ == EV_KEY) && (event.code == BTN_TOUCH) {
                    if event.value == 1 {
                        println!("косание начало");
                        touchpad_pressed = true;
                    }
                    if event.value == 0 {
                        println!("косание конец");
                        touchpad_pressed = false;
                        position_x = -1;
                        position_y = -1;
                    }
                }
                // положение X
                if (event.type_ == EV_ABS) && (event.code == ABS_X) {
                    //println!("X = {:?}", event.value);
                    if position_x == -1{
                        position_x = event.value;
                    }

                }
                // положение Y
                if (event.type_ == EV_ABS) && (event.code == ABS_Y) {
                    //println!("Y = {:?}", event.value);
                    if position_y == -1{
                        position_y = event.value;
                    } else{
                        if position_x > 3500 {
                            if event.value < position_y - 5 {
                                if brightness < 540 {
                                    brightness = brightness + 5;
                                }

                                contents = brightness.to_string();
                                file.write_all(contents.as_bytes())?;

                                println!("brightness: {}", brightness);
                                position_y = event.value;
                            }
                            if event.value > position_y + 5 {
                                if brightness > 40 {
                                    brightness = brightness - 5;
                                }

                                contents = brightness.to_string();
                                file.write_all(contents.as_bytes())?;

                                println!("brightness: {}", brightness);
                                position_y = event.value;
                            }
                        }
                    }
                }

            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
            _ => {}
        }
    }
}
