extern crate minidbg;
extern crate nix;

use std::env;
use std::process;
use nix::sys::wait::*;
use nix::unistd::*;
use nix::sys::ptrace;
use std::ffi::CString;
use minidbg::Debugger;


fn main() {
    let args: Vec<String> = env::args().collect();
    if  args.len() < 2 {
        println!("USAGE: ./minidbg <binary_file>");
        process::exit(-1);
    }
    let prog = args[1].clone();

    match fork().expect("fork failed") {
        ForkResult::Parent {child} => {
            //do something in parent process           
            let mut dbg = Debugger::new(&prog, &child);// {m_prog_name: prog, m_pid: child};
            dbg.run();
        }
        ForkResult::Child => {
            //do smething in child process
            let dir = CString::new(prog.to_string()).unwrap();

            ptrace::traceme();
            execv(
                &dir,
                &[
                    dir.clone()
                ],
                ).expect("execution failed");
        }
    }
}
