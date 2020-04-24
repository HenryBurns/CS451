use std::cmp;
use std::fs::File;
use std::fs::{self};
use std::str;
use std::io::Write;
use std::io::Read;
use std::process;
use std::path::Path;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;


pub enum PPMerr {
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
    let max_size = buffer.len();

    if buffer[0] != 'P' as u8 {
        //eprintln!("first character was {}, not P", buffer[0] as char);
        return Err(PPMerr::InvalidHeader);
    } else if buffer[1] != ('3' as u8) && buffer[1] != ('6' as u8) {
        //eprintln!("second character wasn't 3 or 6, Char: {}", buffer[1] as char);
        return Err(PPMerr::InvalidHeader);
    }

    // Read P6 or P3
    offset += 3;
    if offset >= max_size {
        //eprintln!("This file was too small");
        return Err(PPMerr::InvalidHeader);
    }
    while !(buffer[offset] as char).is_ascii_whitespace() {
        if buffer[offset] < '0' as u8 || buffer[offset] > '9' as u8 {
            //eprintln!("Header width contained non numbers. Val: {}", buffer[offset] as char);
            return Err(PPMerr::InvalidHeader);
        } else {
            width *= 10;
            width += usize::from(buffer[offset] - '0' as u8);

            offset += 1;
            if offset == max_size {
                //eprintln!("This file was too small");
                return Err(PPMerr::InvalidHeader);
            }
        }
    }
    offset += 1;
    if offset == max_size {
        //eprintln!("This file was too small");
        return Err(PPMerr::InvalidHeader);
    }

    // This gets us past width
    while !(buffer[offset] as char).is_ascii_whitespace() {
        if buffer[offset] < '0' as u8 || buffer[offset] > '9' as u8 {
            //eprintln!("Header height contained non numbers. Val: {}", buffer[offset] as char);
            return Err(PPMerr::InvalidHeader);
        } else {
            height *= 10;
            height += usize::from(buffer[offset] - '0' as u8);
            offset += 1;

            if offset == max_size {
                //eprintln!("This file was too small");
                return Err(PPMerr::InvalidHeader);
            }
        }
    }
    offset += 1;
    if offset == max_size {
        //eprintln!("This file was too small");
        return Err(PPMerr::InvalidHeader);
    }

    // This gets us past height
    while !(buffer[offset] as char).is_ascii_whitespace() {
        if buffer[offset] < '0' as u8 || buffer[offset] > '9' as u8 {
            //eprintln!("Pixel format contained non numbers. Val: {}", buffer[offset] as char);
            return Err(PPMerr::InvalidHeader);
        } else {
            offset += 1;
            if offset == max_size {
                //eprintln!("This file was too small");
                return Err(PPMerr::InvalidHeader);
            }
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

fn get_hidden_message(data: &[u8], ansr: &mut Vec<u8>) {
    let mut temp :u8 = 0;
    let mut found_null = false;
    let mut len = data.len();

    // Truncate our data to a lower multiple of 8.
    if len % 8 != 0 {
        len = (len/8) * 8 
    }
    
    // Extract message
    for i in 0..len {
        if i % 8 == 7 {
            temp |= 0b000_0001 & (data[i]);
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
        ansr.clear();
    }
}

fn is_one(c:u8, bit:usize)-> bool {
    if c & (0b1000_0000 >> bit) != 0 {
        return true;
    }
    return false;
}

fn decode_file(map: Arc<Mutex<HashMap<String, String>>>, header: PPMHeader, data: Vec<u8>, filename : String) {
    let mut message : Vec<u8> = Vec::new();
    let start = header.offset;
    get_hidden_message(&data[start..], &mut message);
    if message.len() == 0 {
        eprintln!("{} did not contain a hidden message", filename);
    }

    let hidden = str::from_utf8(&message);

    if  hidden.is_err() {
        eprintln!("{} does not contain UTF8 data encoded inside", filename);
    } else {
        let mut lock = map.lock().unwrap();
        lock.insert(filename, String::from(hidden.unwrap()));
    }
}

pub fn decode_directory(n_threads: usize, dir_path: String){
    let err_str ="There was a problem reading image files from the provided directory";
    let dir = Path::new(&dir_path);
    let mut data = Vec::new();
    let mut files = Vec::new();

    // Thread variables
    let mut threads = Vec::new();

    // HashMap<filename -> data)
    let map : Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    if dir.is_dir() {
        // Iterate through all the files in the specified directory
        // TODO: Make sure this iterates in lexicographical order.
        for entry in fs::read_dir(dir).expect(err_str) {

            let entry = entry.expect("There was a problem Iterating accross the image directory");
            let path = entry.path();
            let filename = String::from(path.to_str().unwrap());

            // Save the filenames for later

            // Read in the contents of the file, so we can parse it. TODO: only read in header.
            read_file(path.to_str().unwrap(), &mut data).expect("We couldn't read the file");
            match get_header(&data) {
                Ok(header) => {
                    files.push(filename.clone());
                    let map_copy = Arc::clone(&map);
                    let data_copy = data.to_vec();

                    if threads.len() != n_threads {

                        threads.push(Some(thread::spawn(move || {
                            decode_file(map_copy, header, data_copy, filename);
                        })));

                    } else {

                        join_all_threads(&mut threads);
                        threads.clear();
                        threads.push(Some(thread::spawn(move || {
                            decode_file(map_copy, header, data_copy, filename);
                        })));
                    }
                },
                Err(_e) => {
                    eprintln!("{} is not a proper formatted PPM file", path.display());
                },
            }
            data.clear();
        }
    } else { // The path is not a directory
        eprintln!("Provide a directory as the image path. You provided \"{}\"", dir.display());
        eprintln!("Note: We also explore all sub directories in the provided path, this may be causing problems");
        return;
    }
    files.sort();

    join_all_threads(&mut threads);
    threads.clear();


    let locked_map = map.lock().unwrap();

    let mut final_message = String::new();
    for file in files {
        final_message.push_str(&locked_map[&file.to_owned()]);
    }

    println!("{}", final_message);
}

fn encode(message: Vec<u8>, data: &mut Vec<u8>, o_file: String, file_num: usize, header: PPMHeader) {
    let start = header.offset;

    let mut offset = 0;
    let mut total = start;

    // Since message includes the null byte, we can just write data to o_file/file_num.
    for c in &message {
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

    // Write null pyte
    for indx in (total)..(total + 8) {
        data[indx] &= 0b1111_1110;
    }

    // Create output file name {0000_0000} file_num.ppm
    let file = Path::new(&format!("{}", o_file)).join(format!("{:0>8}.ppm", file_num.to_string()));

    let mut output = File::create(file).expect("We should be able to write to files in this directory");
    output.write_all(data).expect("We should be able to write our vec to this output file");
    output.flush().unwrap();
}

fn join_all_threads(threads: &mut Vec<Option<thread::JoinHandle<()>>>) {
    for child in threads {
        child.take().unwrap().join().expect("The child thread has panicked, help");
    }
}

fn encode_subdir(message: &Vec<u8>, indx: &mut usize, dir: &Path, o_file: &String, n_threads: usize){
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

            let entry = entry.expect("There was a problem Iterating accross the image directory");
            let path = entry.path();
            let filename = String::from(path.to_str().unwrap());

            // Save the filenames for later

            // Read in the contents of the file, so we can parse it. TODO: only read in header.
            read_file(path.to_str().unwrap(), &mut data).expect("We couldn't read the file");
            match get_header(&data) {
                Ok(header) => {
                    files.push(filename.clone());
                    // We need to copy over the message, and then encode it in our file.
                    eprintln!("{} is a valid ppm file.", filename);
                    let num_bytes = (header.height * header.width)/8;
                    let end = cmp::min(*indx + num_bytes - 1, max_size);
                    let new_message = message[*indx..end].to_vec();
                    let new_ofile = o_file.clone();
                    let mut new_data =  data.to_vec();

                    if threads.len() != n_threads {
                        threads.push(Some(thread::spawn(move || {
                            encode(new_message, &mut new_data, new_ofile, file_num, header)
                        })));
                    } else {
                        join_all_threads(&mut threads);
                        threads.clear();
                        threads.push(Some(thread::spawn(move || {
                            encode(new_message, &mut new_data, new_ofile, file_num, header)
                        })));
                    }

                    *indx = end;
                    file_num += 1;
                },
                Err(PPMerr::InvalidHeader) => {
                    eprintln!("{} is not a proper formatted PPM file", path.display());
                },
            }

            if *indx == max_size { // We've written everything including the null byte.
                join_all_threads(&mut threads); 
                return;
            }
            data.clear();
        }
    } else { // The path is not a directory
        eprintln!("Provide a directory as the image path. You provided \"{}\"", dir.display());
        eprintln!("Note: We also explore all sub directories in the provided path, this may be causing problems");
        return;
    }

    join_all_threads(&mut threads);
    threads.clear();

    if *indx == max_size {
        return;
    } else { // This is the case where the message is too big for all the files
        eprintln!("Our message was too big, let's encode copies of the last file");
        // We can use data, and files.last() to get the image data and filename of the previous file
        let filename = files.last().expect("We should have PPM files in the input directory");
        let path = Path::new(&filename);
        while *indx != max_size {
            // Make a copy of some image and hide more data.
            read_file(path.to_str().clone().unwrap(), &mut data).expect("We couldn't read the file");
            match get_header(&data) {
                Ok(header) => {
                    // We need to copy over the message, and then encode it in our file.
                    let num_bytes = (header.height * header.width)/8;
                    let end = cmp::min(*indx + num_bytes- 1, max_size);
                    let new_message = message[*indx..end].to_vec();
                    let new_ofile = o_file.clone();
                    let mut new_data =  data.to_vec();

                    if threads.len() != n_threads { // Spawn more threads
                        threads.push(Some(thread::spawn(move || {
                            encode(new_message, &mut new_data, new_ofile, file_num, header)
                        })));
                    } else { // Replace existing threads
                        join_all_threads(&mut threads);
                        threads.clear();
                        threads.push(Some(thread::spawn(move || {
                            encode(new_message, &mut new_data, new_ofile, file_num, header)
                        })));
                    }

                    *indx = end;
                    file_num += 1;
                },
                Err(_e) => {
                    eprintln!("The file {} is not a proper formatted PPM file, aborting", path.display());
                    process::abort();
                },
            }
        }
    }
    
    return;
}

pub fn encode_directory(n_threads: usize, message_path: String, dir_path: String, output_path: String) {
    // 1. Read in the message.
    let mut message : Vec<u8> = Vec::new();
    //let mut file = File::create(output_path).expect("There were problems creating the output file");
    let dir = Path::new(&dir_path);
    read_file(&message_path, &mut message).expect(&format!("We weren't able to read the message from file {}", message_path));

    // 2. iterate through all valid images, and read in the header.
    let mut indx: usize = 0;
    encode_subdir(&message,&mut indx, dir, &output_path, n_threads)
}


