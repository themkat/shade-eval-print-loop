use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::Instant,
};

use image::RgbaImage;
use nalgebra::{Matrix4, RowVector4};
use steel::{
    SteelVal,
    parser::ast::IteratorExtensions,
    rvals::IntoSteelVal,
    steel_vm::{engine::Engine, register_fn::RegisterFn},
};
use steel_derive::Steel;

use crate::command::{RenderCommand, StateUpdateCommand, UniformValue};

/// The scheme process' information on the state of the renderer.
#[derive(Default)]
struct RenderState {
    // width, height
    screen_size: (u32, u32),
}

/// Scheme REPL running as a process over the network on port 42069. Sends messages on a channel.
pub struct NetworkScheme {
    scheme_vm: Engine,

    /// Whether previous expression was an error
    prev_was_error: bool,
}

impl NetworkScheme {
    /// The only user facing function. Starts a network process and runs the main loop. Blocks, so recommended to run this in its own thread.
    pub fn main_loop(mut self) {
        // input and output ports for the repl thread
        let repl_channels: Arc<Mutex<Vec<(Receiver<String>, Sender<String>)>>> =
            Arc::new(Mutex::new(Vec::new()));

        {
            let repl_channels = Arc::clone(&repl_channels);
            thread::spawn(move || {
                let listener =
                    TcpListener::bind("127.0.0.1:42069").expect("Could not bind to port 42069!");

                for stream in listener.incoming() {
                    if let Ok(mut stream) = stream {
                        // create a command port to send to the repl
                        let (repl_input_sender, repl_input_receiver) = channel();
                        let (repl_output_sender, repl_output_receiver) = channel();

                        // Add our own communication channels to the REPLs list of users
                        {
                            repl_channels
                                .lock()
                                .unwrap()
                                .push((repl_input_receiver, repl_output_sender));
                        }

                        let mut reader = BufReader::new(stream.try_clone().unwrap());

                        // TODO: maybe allow multiple requests instead of blocking on first connection? Reconnects etc. might be messy here
                        loop {
                            // write prompt
                            stream.write_all(b"> ").unwrap();

                            let mut buffer = String::new();
                            if let Ok(bytes_read) = reader.read_line(&mut buffer) {
                                if bytes_read == 0 {
                                    continue;
                                }

                                // run the command
                                repl_input_sender.send(buffer).unwrap();
                                let result = repl_output_receiver.recv().unwrap();

                                //let result = self.eval(buffer);
                                stream.write_all(&result.into_bytes()).unwrap();

                                stream.flush().unwrap();
                            }
                        }
                    }
                }
            });
        }

        // main thread repl and dynamic updates loop
        let mut prev_time = Instant::now();
        loop {
            // TODO: tunable interval for older computers?
            if (Instant::now() - prev_time).as_millis() >= 50 {
                self.run_dynamic_updates().unwrap();
                prev_time = Instant::now();
            }

            for (input, output) in repl_channels.lock().unwrap().iter() {
                if let Ok(msg) = input.try_recv() {
                    let result = self.eval(msg);
                    output.send(result).unwrap();
                }
            }
        }
    }

    pub fn new_env(
        input_port: Receiver<StateUpdateCommand>,
        output_port: Sender<RenderCommand>,
    ) -> Self {
        let mut scheme_vm = Engine::new();
        let start_time = Instant::now();

        scheme_vm.register_fn("set-uniform!", move |name: String, value: SteelVal| {
            // TODO: better error handling!
            output_port
                .send(RenderCommand::SetUniform(
                    name.clone(),
                    match value {
                        SteelVal::NumV(num) => UniformValue::Float(num as f32),
                        SteelVal::ListV(list) => {
                            // TODO: make prettier. This is fucking ugly
                            let all_nums: Vec<Option<f32>> = list
                                .iter()
                                .map(|elem| match elem {
                                    SteelVal::NumV(num) => Some(*num as f32),
                                    _ => None,
                                })
                                .collect();
                            if all_nums.len() == 3 && all_nums.iter().all(|elem| elem.is_some()) {
                                UniformValue::Vector3(
                                    all_nums[0].unwrap(),
                                    all_nums[1].unwrap(),
                                    all_nums[2].unwrap(),
                                )
                            } else {
                                return Err(
                                    "uniform only support coercing 3 element lists to vec3",
                                );
                            }
                        }
                        SteelVal::Custom(val) => {
                            if let Some(matrix) = val.borrow().as_any_ref().downcast_ref::<Matrix>()
                            {
                                UniformValue::Matrix(matrix.into())
                            } else if let Some(texture) =
                                val.borrow().as_any_ref().downcast_ref::<Texture>()
                            {
                                UniformValue::RgbaTexture2D(texture.image.clone())
                            } else {
                                unreachable!("Should never happen")
                            }
                        }
                        _ => return Err("thats a paddlin"),
                    },
                ))
                .unwrap();
            Ok(())
        });

        // dynamic uniform
        // defining a global variable, which seems to be the easiest way while avoiding ownership and thread deadlocks.
        // TODO: prettify so code is easier to read
        scheme_vm
            .run("(define DYNAMIC_UNIFORM_TABLE (hash))".to_string())
            .expect("Could should NOT fail");
        scheme_vm.run("(define (set-dynamic-uniform! name func) (set! DYNAMIC_UNIFORM_TABLE (hash-insert DYNAMIC_UNIFORM_TABLE name func)))".to_string()).expect("Could should NOT fail");
        // TODO: return an error if name don't exist
        scheme_vm.run("(define (delete-dynamic-uniform! name) (set! DYNAMIC_UNIFORM_TABLE (hash-remove DYNAMIC_UNIFORM_TABLE name)))".to_string()).expect("Should NOT fail!");

        // standard library matrix functions
        // TODO: should we support other matrices than 4x4?
        scheme_vm.register_type::<Matrix>("matrix?");
        scheme_vm.register_fn("matrix", Matrix::new);

        // get the elapsed time in seconds (floating point)
        scheme_vm.register_fn("get-elapsed-time", move || {
            (Instant::now() - start_time).as_secs_f32()
        });

        // simple texture loading
        scheme_vm.register_type::<Texture>("texture?");
        scheme_vm.register_fn("load-texture", |filename: String| Texture::new(filename));

        // start a background process that listens to updates from renderer
        // TODO: maybe this setup fits better as a separate method being called in main loop?
        let render_state = Arc::new(Mutex::new(RenderState::default()));
        let render_state_clone = Arc::clone(&render_state);
        thread::spawn(move || {
            let render_state = render_state_clone;
            loop {
                if let Ok(command) = input_port.recv() {
                    match command {
                        StateUpdateCommand::ScreenSizeChanged(width, height) => {
                            render_state.lock().unwrap().screen_size = (width, height);
                        }
                    }
                }
            }
        });

        // function to fetch screen size information
        scheme_vm.register_fn("screen-size", move || {
            render_state.lock().unwrap().screen_size
        });

        Self {
            scheme_vm,
            prev_was_error: false,
        }
    }

    /// Evaluates a scheme expression and returns the return value as a String.
    // TODO: handle stdout somehow!
    fn eval(&mut self, expression: String) -> String {
        let return_value = self.scheme_vm.run(expression);

        match return_value {
            Ok(return_value) => {
                self.prev_was_error = false;

                // we are only interested in the last evaluated expression.
                // no need to print all of them.
                // Void is also a SteelVal type :)
                let result = return_value.last().unwrap_or(&SteelVal::Void);

                // Ugly hack to use our own Display implementations for custom types
                // TODO: refactor with match
                if let SteelVal::Custom(val) = result {
                    if let Some(matrix) = val.borrow().as_any_ref().downcast_ref::<Matrix>() {
                        format!("{}\n", matrix)
                    } else {
                        format!("{}\n", result)
                    }
                } else {
                    format!("{}\n", result)
                }
            }
            Err(err) => {
                self.prev_was_error = true;
                // prints the error to console for debugging purposes
                eprintln!("[ERROR] {}", err);

                format!("ERROR: Evaluation failed: {}\n", err)
            }
        }
    }

    /// Runs one iteration of dynamic updates. Meant to be used in a loop and called regularly.
    fn run_dynamic_updates(&mut self) -> Result<(), String> {
        let dynamic_uniform_table = self
            .scheme_vm
            .extract::<HashMap<String, SteelVal>>("DYNAMIC_UNIFORM_TABLE")
            .expect("table should always exist");

        for (name, val) in &dynamic_uniform_table {
            if let SteelVal::Closure(_) = val {
                // TODO: better error handling here.. Lots of internal methods used. Electric bogaloo
                let result = self
                    .scheme_vm
                    .call_function_with_args(val.clone(), vec![])
                    .unwrap();
                self.scheme_vm
                    .call_function_by_name_with_args(
                        "set-uniform!",
                        vec![name.clone().into_steelval().unwrap(), result],
                    )
                    .unwrap();
            } else {
                return Err(
                    "Second argument to set-dynamic-uniform! should be a lambda.".to_string(),
                );
            }
        }

        Ok(())
    }
}

// Custom types to let me define Display trait and custom operations
#[derive(Debug, Clone, PartialEq, Steel)]
struct Matrix {
    elements: Vec<Vec<f32>>,
}

impl Matrix {
    fn new(row1: Vec<f32>, row2: Vec<f32>, row3: Vec<f32>, row4: Vec<f32>) -> Result<Self, String> {
        if row1.len() == 4
            && row1.len() == row2.len()
            && row2.len() == row3.len()
            && row3.len() == row4.len()
        {
            Ok(Matrix {
                elements: vec![row1, row2, row3, row4],
            })
        } else {
            Err("Invalid dimensions".to_string())
        }
    }

    // TODO: maybe a new method that takes ints as well? Makes it super convenient to avoid writing the .0 if we don't have any decimals. Also makes it more similar to the debug prints
}

impl From<&Matrix> for Matrix4<f32> {
    fn from(value: &Matrix) -> Self {
        // length assertion is handled in constructor
        let row1 = RowVector4::new(
            value.elements[0][0],
            value.elements[0][1],
            value.elements[0][2],
            value.elements[0][3],
        );
        let row2 = RowVector4::new(
            value.elements[1][0],
            value.elements[1][1],
            value.elements[1][2],
            value.elements[1][3],
        );
        let row3 = RowVector4::new(
            value.elements[2][0],
            value.elements[2][1],
            value.elements[2][2],
            value.elements[2][3],
        );
        let row4 = RowVector4::new(
            value.elements[3][0],
            value.elements[3][1],
            value.elements[3][2],
            value.elements[3][3],
        );

        Matrix4::from_rows(&[row1, row2, row3, row4])
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let space_separate =
            |vec: &Vec<f32>| -> String { vec.iter().map(|elem| elem.to_string()).join(" ") };

        write!(
            f,
            "(({})\n ({})\n ({})\n ({}))",
            space_separate(&self.elements[0]),
            space_separate(&self.elements[1]),
            space_separate(&self.elements[2]),
            space_separate(&self.elements[3])
        )
    }
}

/// Simple wrapper type for textures.
#[derive(Clone, Steel)]
struct Texture {
    image: RgbaImage,
}

impl Texture {
    fn new(filename: String) -> Result<Self, String> {
        let texture = image::open(filename).map_err(|err| err.to_string())?;

        Ok(Texture {
            image: texture.to_rgba8(),
        })
    }
}

/// Tests for any extensions the networked scheme adds to its environment.
#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel},
        thread,
        time::Duration,
    };

    use nalgebra::Matrix4;

    use crate::{
        command::{RenderCommand, StateUpdateCommand, UniformValue},
        scheme::Matrix,
    };

    use super::NetworkScheme;

    struct TestHarness {
        state: NetworkScheme,
        // receiver channel used to test that the NetworkScheme sends correct render data
        render_receiver: Receiver<RenderCommand>,
        state_sender: Sender<StateUpdateCommand>,
    }

    // A simple test harness to make the test functions easier to read and maintain
    impl TestHarness {
        fn new() -> Self {
            let (render_sender, render_receiver) = channel::<RenderCommand>();
            let (state_sender, other_receiver) = channel::<StateUpdateCommand>();

            let state = NetworkScheme::new_env(other_receiver, render_sender);

            TestHarness {
                state,
                render_receiver,
                state_sender,
            }
        }

        fn get_last_event(&mut self) -> Result<RenderCommand, RecvTimeoutError> {
            self.render_receiver
                .recv_timeout(Duration::from_micros(500))
        }
    }

    #[test]
    fn matrix_test() {
        let mut testharness = TestHarness::new();
        testharness
            .state
            .eval("(matrix '(1.0) '(1.0 2.0) '(2.0))".to_string());
        assert!(testharness.state.prev_was_error);
        testharness
            .state
            .eval("(matrix '(1.0 2.0 3.0) '(1.0 2.0 2.0) '(2.0 1.0 2.0))".to_string());
        assert!(testharness.state.prev_was_error);

        testharness.state.eval("(define my-matrix (matrix '(1.0 2.0 3.0 4.0) '(1.0 2.0 2.0 2.0) '(2.0 1.0 2.0 1.0) '(3.0 4.0 5.0 6.0)))".to_string());
        assert!(!testharness.state.prev_was_error);

        let matrix = testharness.state.scheme_vm.extract("my-matrix").unwrap();
        assert_eq!(
            Matrix::new(
                vec![1.0, 2.0, 3.0, 4.0],
                vec![1.0, 2.0, 2.0, 2.0],
                vec![2.0, 1.0, 2.0, 1.0],
                vec![3.0, 4.0, 5.0, 6.0]
            )
            .unwrap(),
            matrix
        );

        let result = testharness.state.eval("my-matrix".to_string());
        assert_eq!(
            "((1 2 3 4)\n (1 2 2 2)\n (2 1 2 1)\n (3 4 5 6))\n".to_string(),
            result
        );
    }

    #[test]
    fn uniform_value_test() {
        // relevant channel should receive the uniform value when lisp function is called
        let mut testharness = TestHarness::new();
        testharness
            .state
            .eval("(set-uniform! \"my_pi\" 3.14)".to_string());

        let command = testharness.get_last_event();

        assert!(!testharness.state.prev_was_error);
        // TODO: float rounding errors?
        assert_eq!(
            Ok(RenderCommand::SetUniform(
                "my_pi".to_string(),
                UniformValue::Float(3.14)
            )),
            command
        );
    }

    #[test]
    fn uniform_vec3_test() {
        // 3 element float lists should be coerced into vec3 in glsl
        let mut testharness = TestHarness::new();

        testharness
            .state
            .eval("(set-uniform! \"my_vec\" '(1.0 2.0 4.0))".to_string());

        let command = testharness.get_last_event();
        assert!(!testharness.state.prev_was_error);
        // TODO: float rounding errors?
        assert_eq!(
            Ok(RenderCommand::SetUniform(
                "my_vec".to_string(),
                UniformValue::Vector3(1.0, 2.0, 4.0)
            )),
            command
        );
    }

    #[test]
    fn uniform_matrix_test() {
        // relevant channel should receive the uniform value when lisp function is called
        let mut testharness = TestHarness::new();
        testharness.state.eval("(set-uniform! \"some_matrix\" (matrix '(0.0 1.0 2.0 3.0) '(4.0 5.0 6.0 7.0) '(8.0 9.0 10.0 11.0) '(12.0 13.0 14.0 15.0)))".to_string());

        let command = testharness.get_last_event();

        assert!(!testharness.state.prev_was_error);
        // TODO: float rounding errors?
        assert_eq!(
            Ok(RenderCommand::SetUniform(
                "some_matrix".to_string(),
                UniformValue::Matrix(Matrix4::from_row_slice(&[
                    0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
                    15.0
                ]))
            )),
            command
        );
    }

    #[test]
    fn screen_size_state_update_test() {
        let mut testharness = TestHarness::new();

        testharness
            .state_sender
            .send(StateUpdateCommand::ScreenSizeChanged(250, 820))
            .unwrap();

        // TODO: sometimes flaky on GA. Maybe I could use a fixed size sync_channel instead?
        // sleep two seconds for the background thread to possibly catch up
        thread::sleep(Duration::from_secs(2));

        let result = testharness.state.eval("(screen-size)".to_string());
        assert!(!testharness.state.prev_was_error);
        // no cons cells, so list instead
        assert_eq!("(250 820)\n".to_string(), result);
    }

    #[test]
    fn dynamic_uniform_float_test() {
        let mut testharness = TestHarness::new();

        testharness.state.eval("(define global-val 1.0)
                                (set-dynamic-uniform! \"my-uniform\" (lambda () (set! global-val (+ global-val 2.0)) global-val))".to_string());

        assert!(!testharness.state.prev_was_error);

        // run a single iteration of dynamic updates
        testharness.state.run_dynamic_updates().unwrap();

        let event = testharness.get_last_event();
        assert_eq!(
            Ok(RenderCommand::SetUniform(
                "my-uniform".to_string(),
                UniformValue::Float(3.0)
            )),
            event
        )
    }

    // TODO: ints! There is some handling of plain integers. Useful in some situations in glsl

    // TODO: maybe make some rules for dynamic uniforms to avoid too many pitfalls...
    //       maybe time limit or detection for if stuck?
    //    - arguments == 0, only that allowed
}
