use {
    png,
    std::{
        env,
        fs::{self, File},
        io::{Cursor, Read, Write},
        process,
    },
};

const MAX_PIXEL_VAL: u16 = 3 * 255;

fn compare(val: u16, invert: bool) -> bool {
    if invert {
        val > MAX_PIXEL_VAL / 2
    } else {
        val < MAX_PIXEL_VAL / 2
    }
}

fn print_usage() {
    println!("\nUsage:");
    println!("{} file [invert]", env::args().nth(0).unwrap());
}

fn process_image<I: Into<String>, O: Into<String>>(path_in: I, path_out: O, invert_bits: bool) {
    let decoder = png::Decoder::new(File::open(path_in.into()).expect("Could not open file"));
    let (info, mut reader) = decoder.read_info().expect("Failed to read png info");

    let mut buf = vec![0u8; info.buffer_size()];

    if buf.len() % 3 != 0 {
        panic!("Invalid length");
    }

    reader.next_frame(&mut buf).expect("Could not read frame");

    let mut buf16 = vec![0u16; buf.len() / 3];
    for i in 0..(buf.len() / 3) {
        buf16[i] = buf[(i * 3 + 0) as usize] as u16
            + buf[(i * 3 + 1) as usize] as u16
            + buf[(i * 3 + 2) as usize] as u16;
    }

    let mut buf2d = vec![vec![0u16; info.width as usize]; info.height as usize];

    for i in 0..info.height as usize {
        for j in 0..info.width as usize {
            buf2d[i][j] = buf16[((i * info.width as usize) + j) as usize];
        }
    }

    let mut of = File::create(path_out.into()).expect("Failed to create output file");
    of.write(
        format!(
            "#include <inttypes.h>\n\n#define IMG_WIDTH {}\n#define IMG_HEIGHT {}\n\nuint8_t img[IMG_HEIGHT][IMG_WIDTH] = {{\n",
            info.width, info.height
        )
        .as_str()
        .as_bytes(),
    )
    .expect("Failed to write to file");

    for y in 0..buf2d.len() {
        of.write("    { ".as_bytes())
            .expect("Failed to write to file");
        for x in 0..info.width as usize {
            print!(
                "{}",
                if compare(buf2d[y][x], invert_bits) {
                    "_"
                } else {
                    "X"
                }
            );
            of.write(
                format!(
                    "{}{}",
                    if compare(buf2d[y][x], invert_bits) {
                        1
                    } else {
                        0
                    },
                    if x + 1 == info.width as usize {
                        ""
                    } else {
                        ", "
                    }
                )
                .as_str()
                .as_bytes(),
            )
            .expect("Failed to write to file");
        }
        of.write(
            format!(
                " }}{}",
                if y + 1 == info.height as usize {
                    "\n"
                } else {
                    ",\n"
                }
            )
            .as_str()
            .as_bytes(),
        )
        .expect("Failed to write to file");
        println!();
    }

    of.write("};\n".as_bytes())
        .expect("Failed to write to file");
}

fn main() {
    println!("PNG to bits");

    process_image("aa.png", "bb.h", false);

    // println!("{:#?}", png_img);
}
