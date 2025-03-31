use std::{io::{BufRead as _, BufReader, BufWriter, Read, Write}, net::TcpListener};

use shade_eval_print_loop::init;
use steel::{steel_vm::{engine::Engine, register_fn::RegisterFn}, SteelVal};

fn boner() -> String {
    String::from("boner")
}

// TODO: decouple the scheme stuff with the rest of the code. Command pattern. Our own internal language.
//       only put things into the application with pre-defined ports plus uniform variables? (set-dynamic-uniform-variable! "" (lambda () (our-encapsulated-logic)))
//  app can then have a listener loop for these variables and act accordingly
// (register-callback! action (lambda () ...)) - where action can be (kbd)

// TODO: maybe we could also use this tool to take screenshots? To export for later use? set a higher resolution if we want to etc. Maybe useful for generating dudv maps, normal maps for height etc. 

fn main() {
    // TODO: make a tcp server that clients can connect to

    init();
    
    if 1 == 1 {
        return;
    }

    
    let mut scheme_vm = Engine::new();
    scheme_vm.register_fn("boner", &boner);
    // TODO: can we subscribe to changes in variables etc. in any way?
    //       maybe implement my own? after each evaluation we fetch values in a list of values to subscribe to. Then update them were applicable. Might be uniforms, or other thingies. Those that need changing frame to frame that is. how to do this in a clean nice way? maybe we can have a callback system of sorts here? 
    
    
    let listener = TcpListener::bind("127.0.0.1:42069").expect("Could not bind to port 42069!");
    
    for stream in listener.incoming() {
        
        if let Ok(mut stream) = stream {
            // TODO: should we handle connections in a loop?
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            
            // TODO: exit status
            loop {
                let mut buffer = String::new();
                // TODO: why does this not read line by line? or at least not read each entry? Why only once???
                if let Ok(bytes_read) = reader.read_line(&mut buffer) {

                    if bytes_read == 0 {
                        continue;
                    }
                    
                    //println!("Received: {}", buffer);
                    
                    if "(exit)".to_string() == buffer {
                        break;
                    }

                    // run the command
                    // TODO: can we avoid the cloning?!?
                    let return_value = scheme_vm.run(buffer);
                    
                    match return_value {
                        Ok(return_value) => {
                            // we are only interested in the last evaluated expression.
                            // no need to print all of them.
                            // Void is also a SteelVal type :)
                            let result = &return_value.last().unwrap();

                            // TODO: this ignores stdout :/ would be good if the process could use stuff like display and newline to print stuff as well :/ functions with reports etc. :/ 
                            let formatted_result = format!("{}\n", result);
                            // TODO: do something with result on our end. Process results etc. Fetch global statuses of objects we want to follow etc. Maybe a global variable needs update on the server side.
                            // TODO: rust channels
                            stream.write_all(&formatted_result.into_bytes()).unwrap();
                        },
                        Err(_) => {
                            stream.write_all(b"ERROR: Evaluation failed\n").unwrap();
                        },
                    }
                    
                    stream.write_all(b"> ").unwrap();
                    stream.flush().unwrap();
                }
            }
        }
    }
}
