use {
    clap, png,
    std::{env, fs::File, io::Write, path::PathBuf},
};

#[derive(Debug)]
struct Options {
    input: PathBuf,
    output: PathBuf,
    compact: bool,
    flip: bool,
    invert: bool,
    print: bool,
    debug: bool,
    threshold: (u8, u8, u8),
}

fn compare(rgb: (u8, u8, u8), options: &Options) -> bool {
    let mut ret =
        rgb.0 > options.threshold.0 && rgb.1 > options.threshold.1 && rgb.2 > options.threshold.2;
    if options.invert {
        ret = !ret;
    }
    ret
}

fn process_image(options: &Options) {
    let decoder = png::Decoder::new(File::open(&options.input).expect("Could not open file"));
    let (info, mut reader) = decoder
        .read_info()
        .expect("Failed to read png info. Invalid PNG?");

    let mut buf = vec![0u8; info.buffer_size()];

    if buf.len() % 3 != 0 {
        panic!("Invalid length");
    }

    reader.next_frame(&mut buf).expect("Could not read frame");

    let mut buf16 = vec![(0u8, 0u8, 0u8); buf.len() / 3];
    for i in 0..(buf.len() / 3) {
        buf16[i] = (
            buf[(i * 3 + 0) as usize],
            buf[(i * 3 + 1) as usize],
            buf[(i * 3 + 2) as usize],
        );
    }

    let mut buf2d = vec![vec![(0u8, 0u8, 0u8); info.width as usize]; info.height as usize];

    for i in 0..info.height as usize {
        for j in 0..info.width as usize {
            buf2d[i][j] = buf16[((i * info.width as usize) + j) as usize];
        }
    }

    let mut of = File::create(&options.output).expect("Failed to create output file");
    of.write(
        format!(
            "#include <inttypes.h>\n\n#define IMG_WIDTH {}\n#define IMG_HEIGHT {}\n\nstatic const uint8_t img[IMG_HEIGHT][IMG_WIDTH] = {{\n",
            if options.compact { info.width / 8 } else { info.width }, info.height
        )
        .as_str()
        .as_bytes(),
    )
    .expect("Failed to write to file");

    for y in 0..buf2d.len() {
        of.write("    { ".as_bytes())
            .expect("Failed to write to file");
        if options.compact {
            for x in 0..((info.width / 8) as usize) {
                let mut byte = 0u8;
                if options.flip {
                    for i in (0..=7).rev() {
                        byte |= (if compare(buf2d[y][(x * 8) + 7 - i], &options) {
                            if options.print {
                                print!("X");
                            }
                            1
                        } else {
                            if options.print {
                                print!(" ");
                            }
                            0
                        }) << i;
                    }
                } else {
                    for i in 0..=7 {
                        byte |= (if compare(buf2d[y][(x * 8) + i], &options) {
                            if options.print {
                                print!("X");
                            }
                            1
                        } else {
                            if options.print {
                                print!(" ");
                            }
                            0
                        }) << i;
                    }
                }
                of.write(
                    format!(
                        "{:3}{}",
                        byte,
                        if x + 1 == (info.width / 8) as usize {
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
        } else {
            for x in 0..info.width as usize {
                if options.print {
                    print!(
                        "{}",
                        if compare(buf2d[y][x], options) {
                            " "
                        } else {
                            "X"
                        }
                    );
                }
                of.write(
                    format!(
                        "{}{}",
                        if compare(buf2d[y][x], options) { 1 } else { 0 },
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
        if options.print {
            println!();
        }
    }

    of.write("};\n".as_bytes())
        .expect("Failed to write to file");
}

fn main() {
    let yaml = clap::load_yaml!("../cli.yml");
    let matches = clap::App::from_yaml(yaml)
        .author(clap::crate_authors!("\n"))
        .version(clap::crate_version!())
        .name(clap::crate_name!())
        .about(clap::crate_description!())
        .get_matches();

    let options = Options {
        input: matches
            .value_of("INPUT")
            .map(PathBuf::from)
            .expect("Input not set"),
        output: matches
            .value_of("OUTPUT")
            .map(PathBuf::from)
            .expect("Output not set"),
        compact: matches.is_present("compact"),
        flip: matches.is_present("flip-bits"),
        invert: matches.is_present("invert"),
        print: matches.is_present("print"),
        debug: matches.is_present("debug"),
        threshold: (
            matches
                .value_of("red")
                .map(|r| r.parse::<u8>().expect("Failed to parse red threshold"))
                .unwrap_or(127),
            matches
                .value_of("green")
                .map(|g| g.parse::<u8>().expect("Failed to parse green threshold"))
                .unwrap_or(127),
            matches
                .value_of("blue")
                .map(|b| b.parse::<u8>().expect("Failed to parse blue threshold"))
                .unwrap_or(127),
        ),
    };

    if options.debug {
        println!("{:#?}", options);
    }

    process_image(&options);

    println!("Done! Output saved to {:?}", options.output);
}
