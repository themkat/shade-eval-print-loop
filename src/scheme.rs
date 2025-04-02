

// TODO: maybe have a method to get values updated since last call?

use std::{io::BufReader, io::BufRead, io::Write, net::TcpListener, thread};

use steel::steel_vm::{engine::Engine, register_fn::RegisterFn};

/// Scheme REPL running as a process over the network on port 42069. Sends messages on a channel.
pub struct NetworkScheme;

impl NetworkScheme {

    // TODO: how to structure this the best?
    pub fn main_loop() {
        thread::spawn(|| {
            // stops automatically once parent stops
            // TODO: create tcp connection
            let mut scheme_vm = Engine::new();
            //scheme_vm.register_fn("boner", &boner);

            // TODO: maybe we now can have a few of these with some channels, hashmaps or something similar? Then we can save values to our global scope and do all sorts of things
            scheme_vm.register_fn("hithere", || {
                println!("closure");
                1
            });

            let listener = TcpListener::bind("127.0.0.1:42069").expect("Could not bind to port 42069!");
    
            for stream in listener.incoming() {
                
                if let Ok(mut stream) = stream {
                    // TODO: should we handle connections in a loop?
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    
                    // TODO: exit status
                    loop {
                        // write prompt
                        stream.write_all(b"> ").unwrap();
                        
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
                                    
                                    // TODO: this ignores stdout :/ would be good if the process could use stuff like display and newline // TODO: o print stuff as well :/ functions with reports etc. :/ 
                                    let formatted_result = format!("{}\n", result);
                                    // TODO: do something with result on our end. Process results etc. Fetch global statuses of objects we want to follow etc. Maybe a global variable needs update on the server side.
                                    // TODO: rust channels
                                    stream.write_all(&formatted_result.into_bytes()).unwrap();
                                },
                                Err(_) => {
                                    stream.write_all(b"ERROR: Evaluation failed\n").unwrap();
                                },
                            }
                            
                            stream.flush().unwrap();
                        }
                    }
                }
            }
        });
    }
    
}

// functions exposed from scheme
// TODO: get-elapsed-time-seconds etc.
//       set-uniform!
//       set-dynamic-uniform! with lambda


// TODO: tests using channels? to verify assumptions? or can we instead use a hashmap or something?
