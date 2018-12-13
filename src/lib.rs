extern crate nix;
extern crate linenoise;

use std::process;
use nix::sys::wait::*;
use nix::unistd::*;
use nix::sys::ptrace;
use nix::libc;
use std::ffi::CString;
use std::ptr;
use std::collections::HashMap;
use std::u32;

pub struct Debugger {
    pub m_prog_name: String,
    pub m_pid: Pid,
    m_breakpoints: HashMap<u32, Breakpoint>,
}

impl Debugger {
    pub fn run(&mut self) {
        let options = 0;
        let pid = self.m_pid;
        waitpid(pid, None).expect("wait pid failed");

        loop{
            let line = linenoise::input("dbg> ");

            match line {
                None => {break}
                Some(input) => {
                    self.handle_command(&input);
                    linenoise::history_add(input.as_ref());
                },
            }
        }
    }

    pub fn new(prog_name: &String, pid: &Pid) -> Debugger {
        let m_prog_name = prog_name.clone();
        let m_pid = pid.clone();
        let m_breakpoints = HashMap::new();

        Debugger{m_prog_name, m_pid, m_breakpoints}
    }

    pub fn set_breakpoint_at_address(&mut self, addr: &u32) {
        println!("Set breakpoint at address 0x{:x}", addr);

        let mut bp = Breakpoint::new(&self.m_pid, &addr);
        bp.enable();
        self.m_breakpoints.insert(*addr, bp);
    }

    fn handle_command(&mut self, line: &String) {
        let args:Vec<&str> = line.split(' ').collect();
        let command = args[0];

        if command == "cont" {
            self.continue_execution();
        }else if command == "break" {
            let addr: u32 = u32::from_str_radix(args[1].trim_left_matches("0x"),16).unwrap();
            self.set_breakpoint_at_address(&addr);
        } else {
            println!("Unknown command\n");
        }
    }


    fn continue_execution(&self) {
        let pid = self.m_pid;

        unsafe{
            ptrace::cont(pid, None);
        }

        waitpid(pid, None).expect("wait pid failed");
    }
}

struct Breakpoint {
    m_pid: Pid,
    m_addr: u32,
    m_enabled: bool,
    m_saved_data: u8,
}

impl Breakpoint {
    fn new(pid: &Pid, addr: &u32) -> Breakpoint{
        let m_pid = pid.clone();
        let m_addr = addr.clone();
        let m_enabled = false;
        let m_saved_data = 0;

        Breakpoint {m_pid, m_addr, m_enabled, m_saved_data}
    }

    fn enable(&mut self) {
        unsafe{
            let pid = self.m_pid;
            let addr = self.m_addr;
            let data = ptrace::ptrace(ptrace::Request::PTRACE_PEEKDATA, pid, addr as *mut libc::c_void, ptr::null_mut()).unwrap();
        
            self.m_saved_data = (data & 0xff) as u8;

            let int3: u64 = 0xcc;
            let int3_with_data = ((data & !0xff) | int3 as i64);
        
            ptrace::ptrace(ptrace::Request::PTRACE_POKEDATA, pid, addr as *mut libc::c_void, int3_with_data as *mut libc::c_void);
        }

        self.m_enabled = true;
    }

    fn disable(&mut self) {
        unsafe {
            let data = ptrace::ptrace(ptrace::Request::PTRACE_PEEKDATA, self.m_pid, self.m_addr as *mut libc::c_void, ptr::null_mut()).unwrap();
        
        let restored_data = ((data & !0xff) | self.m_saved_data as i64);
        
            ptrace::ptrace(ptrace::Request::PTRACE_POKEDATA, self.m_pid, self.m_addr as *mut libc::c_void, restored_data as *mut libc::c_void);
        }

        self.m_enabled = false;
    }

}

















