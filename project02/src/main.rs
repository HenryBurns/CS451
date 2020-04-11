use std::cmp;
use std::fs::File;
use std::fs::{self};
use std::str;
use std::io::{self, Write};
use std::io::Read;
use std::process;
use std::path::Path;
use std::thread;

pub enum PPMerr {
    OK,
    InvalidHeader,
}

struct PPMHeader {

    width : usize,
    height : usize,

    // We also store the offset in the file where the image data starts,
    offset : usize,
}

fn get_header(buffer: &Vec<u8>) -> Result<PPMHeader, PPMerr> {
    let mut offset = 0;
    let mut width : usize = 0;
    let mut height : usize = 0;

    if buffer[0] != 'P' as u8 {
        eprintln!("first character wasn't P");
        return Err(PPMerr::InvalidHeader);
    } else if buffer[1] != ('3' as u8) && buffer[1] != ('6' as u8) {
        eprintln!("second character wasn't 3 or 6, Char: {}", buffer[1] as char);
        return Err(PPMerr::InvalidHeader);
    }

    // Read P6 or P3
    offset += 3;
    while buffer[offset] != ('\n' as u8) && buffer[offset] != (' ' as u8) {
        if buffer[offset] < '0' as u8 || buffer[offset] > '9' as u8 {
            eprintln!("Header width contained non numbers. Val: {}", buffer[offset] as char);
            return Err(PPMerr::InvalidHeader);
        } else {
            width = (width + usize::from('0' as u8 - buffer[offset])) * 10;
            offset += 1;
        }
    }
    offset += 1;

    // This gets us past width
    while buffer[offset] != ('\n' as u8) && buffer[offset] != (' ' as u8) {
        if buffer[offset] < '0' as u8 || buffer[offset] > '9' as u8 {
            eprintln!("Header height contained non numbers. Val: {}", buffer[offset] as char);
            return Err(PPMerr::InvalidHeader);
        } else {
            offset += 1;
            height = (height + usize::from('0' as u8 - buffer[offset])) * 10;
        }
    }
    offset += 1;

    // This gets us past height
    while buffer[offset] != ('\n' as u8) && buffer[offset] != (' ' as u8) {
        if buffer[offset] < '0' as u8 || buffer[offset] > '9' as u8 {
            eprintln!("Pixel format contained non numbers. Val: {}", buffer[offset] as char);
            return Err(PPMerr::InvalidHeader);
        } else {
            offset += 1;
        }
    }
    offset += 1;

    // This should get us past pixel format to the actual data
    return Ok(PPMHeader{ width: width, height : height, offset : offset, });
}

fn read_file(filename: &str, buffer: &mut Vec<u8>) -> std::result::Result<(), std::io::Error> {
    let mut f = File::open(filename)?;
    f.read_to_end(buffer)?;
    buffer.pop(); // We read in 1 extra byte for some reason
    Ok(())
}

fn print_file(data: &Vec<u8>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(data).expect("There was a problem with writing output to stdout");
}

fn get_hidden_message(data: &[u8]) -> Vec<u8> {
    let mut temp :u8 = 0;
    let mut ansr : Vec<u8> = Vec::new();
    let mut found_null = false;
    let mut len = data.len();
    if len % 8 != 0 {
        len = (len/8) * 8 
    }
    for i in 0..len {
        if i % 8 == 7 {
            temp |= 0b000_0001 & (data[i]);
            if !temp.is_ascii() {
                //eprintln!("NOT ASCII");
            }
            if temp == 0 {
                found_null = true;
                break;
            }
            ansr.push(temp);
            temp = 0;
        } else {
            temp |= (0b0000_0001 & (data[i])) <<  7 - (i % 8)
        }
    }
    if !found_null {
        eprintln!("This message wasn't terminated by a null character");
        process::abort();
    }
    return ansr;
}

fn print_hidden_message(filename: &String) {
    let mut data : Vec<u8> = Vec::new();
    read_file(filename, &mut data).expect("We couldn't read the file");
    let header = match get_header(&data) {
        Ok(header) => header,
        Err(_e) => {
            eprintln!("The file {} is a proper formatted PPM file", filename);
            return;
        },
    };
    let start = header.offset;

    data = get_hidden_message(&data[start..]);

    for item in  &data {
        print!("{}", *item as char);
    }
}

fn hide_message(filename: &String, message_filename: &String) {
    let mut data : Vec<u8> = Vec::new();
    let mut message_data : Vec<u8> = Vec::new();
    //let message_str = read_string(message_filename, &mut message_data);
    read_file(message_filename, &mut message_data).expect("We couldn't read the input file");

    let message = match str::from_utf8(&message_data[..]) {
        Ok(v) => String::from(v),
        Err(_v) => panic!("Invalid input provided"),
    };
    read_file(filename, &mut data).expect("We couldn't read the file");
    
    let header = match get_header(&data) {
        Ok(header) => header,
        Err(_e) => {
            eprintln!("The file {} is a proper formatted PPM file", filename);
            return;
        },
    };
    let start = header.offset;

    let mut offset = 0;
    let mut total = start;

    for c in &message_data {
        if ! c.is_ascii() {
            eprintln!("We are reading a non ascii charater");
        }
        for indx in (total)..(total + 8) {
            if is_one(*c, indx-total) {
                data[indx] |= 0b0000_0001;
            } else {
                data[indx] &= 0b1111_1110;
            }
        }
        offset += 8;
        total = start + offset;
    }

    for indx in (total)..(total + 8 ) {
        data[indx] &= 0b1111_1110;
    }

    print_file(&data)
}

fn is_one(c:u8, bit:usize)-> bool {
    if bit > 7 {
        eprintln!("Tried to get a bit beyond the end of this u8");
    }
    if c & (0b1000_0000 >> bit) != 0 {
        return true;
    }
    return false;
}

fn decode_directory(n_threads: usize, dir_path: &String){
}

fn write_filenames(filenames: &Vec<String>, o_file: &String) {
    // List all of the files that we have written to as our output.
    let mut file = File::create(o_file).expect("There were problems creating the output file");

    for iter in filenames {
        write!(file, "{}", iter).expect("We should be able to write to the output file");
    }
}

fn encode(message: &[u8], data: Vec<u8>, o_file: String, file_num: usize) {

}
fn encode_subdir(message: &Vec<u8>, indx: &mut usize, dir: &Path, o_file: &String){
    let err_str ="There was a problem reading image files from the provided directory";
    let max_size = message.len();
    if *indx == max_size {
        return;
    }

    let mut files = Vec::new();
    let mut threads = Vec::new();
    let mut file_num = 0;

    let mut data : Vec<u8> = Vec::new();
    if dir.is_dir() {
        // Iterate through all the files in the specified directory
        // TODO: Make sure this iterates in lexicographical order.
        for entry in fs::read_dir(dir).expect(err_str) {
            if *indx == max_size { // We've written everything including the null byte.
                write_filenames(&files, &o_file);
                return;
            }
            data.clear();
            let entry = entry.expect("There was a problem Iterating accross the image directory");
            let path = entry.path();
            let filename = String::from(path.to_str().unwrap());

            // Save the filenames for later
            files.push(filename.clone());

            // Read in the contents of the file, so we can parse it. TODO: only read in header.
            read_file(path.to_str().unwrap(), &mut data).expect("We couldn't read the file");
            match get_header(&data) {
                Ok(header) => {
                    // We need to copy over the message, and then encode it in our file.
                    let num_bytes = ((header.height * header.width)/8) * 8;
                    let mut end = cmp::min(*indx + num_bytes, max_size);
                    // let new_data : [u8] = message[*indx..end];

                    // TODO: Restrain the number of threads created. Also ask jeremy if the main thread counts.
                    threads.push(thread::spawn(|| {
                        // We want this thread to take a slice of the message
                        // (In the form of a Vec<u8>, and write it out to filename)
                        //encode(new_data, data.clone(), o_file.clone(), file_num)
                    }));
                    *indx += num_bytes;
                    file_num += 1;
                },
                Err(_e) => {
                    eprintln!("The file {} is not a proper formatted PPM file", path.display());
                },
            }
            /*
            if path.is_dir() {
                encode_subdir(message, indx, path)
                }
            }
            */
       }
    } else { // The path is not a directory
        eprintln!("Provide a directory as the image path. You provided \"{}\"", dir.display());
        eprintln!("Note: We also explore all sub directories in the provided path, this may be causing problems");
        return;
    }

    if *indx == max_size {
        return;
    } else { // This is the case where the message is too big for all the files
        while *indx != max_size {
            for entry in fs::read_dir(dir).expect(err_str) {
                /* TODO: something like encode(message[indx..min(indx+num_bytes, max_size - indx), data.clone(), o_file, file_num)
                if path.is_dir() {
                    if encode_subdir(message, indx, path){
                */
            }
        }
    }
    
    write_filenames(&files, &o_file);
    return;
}

fn encode_directory(n_threads: usize, message_path: &String, dir_path: &String, output_path: &String){
    // 1. Read in the message.
    let mut message : Vec<u8> = Vec::new();
    //let mut file = File::create(output_path).expect("There were problems creating the output file");
    let dir = Path::new(dir_path);
    read_file(message_path, &mut message).expect(&format!("We weren't able to read the message from file {}", message_path));
    message.push('\0' as u8);

    // 2. iterate through all valid images, and read in the header.
    let mut indx: usize = 0;
    encode_subdir(&message,&mut indx, dir, & output_path)
}

fn print_usage() {
    eprintln!("Usage1: cargo run <num threads> <Path/To/ImageDir/");
    eprintln!("Usage2: cargo run <num threads> <path/to/message/to/encode> <path/to/Images/>\n\t\t\t<path/to/list/of/images>")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => {
            print_usage()
        },
        3 => {
            match args[1].parse() {
                Err(_e) => {eprintln!("Enter an integer for the number of threads (shown below");
                           print_usage()
                },
                Ok(n_threads) => decode_directory(n_threads, &args[2])
            }
        },
        5 => {
             match args[1].parse() {
                Err(_e) => {eprintln!("Enter an integer for the number of threads (shown below");
                           print_usage()
                },
                Ok(n_threads) => encode_directory(n_threads, &args[2], &args[3], &args[4])
            }
        },
        _ => {
            eprintln!("Run this program with 2 or 4 arguments, as so.");
            print_usage()
        },
    }
}
