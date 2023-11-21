use std::env::{args};
use excel_takehome::excel::Excel;

fn parse_arguments() -> Result<String, &'static str> {
    let usage = "Missing argumnets. Usage cargo run -- [file path]";
    let mut args = args();
    // skip execuable name...
    args.next();
    // parse the first argument as the file path.
    if let Some(file_path) = args.next() {
        return Ok(file_path);
    }
    Err(&usage)
}


fn main() {
    let path = parse_arguments();
    match path {
        Ok(p) => {
            println!("running with file path: {}", p);
            let mut excel = Excel::from_path(&p).expect("File read failed..."); 
            let _ = excel.to_file("out.csv").unwrap();
        },
        Err(message) => println!("{}", message),
    }
}

   

