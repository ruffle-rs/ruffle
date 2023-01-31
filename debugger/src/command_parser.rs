use crate::command::Command;
use ruffle_core::debug::debug_value::DValue;

/// Strip and match
/// Checks if `s` starts with `t`, stripping surrounding whitespace
fn smatch<'a, 'b: 'a>(s: &'a str, t: &'b str) -> Option<&'a str> {
    let s = s.trim_start();
    if let Some(stripped) = s.strip_prefix(t) {
        Some(stripped.trim_start())
    } else {
        None
    }
}

/// Parse a value
fn parse_value(s: &str) -> Option<DValue> {
    if let Ok(v) = s.parse::<i32>() {
        Some(DValue::Int(v))
    } else if let Ok(v) = s.parse::<f64>() {
        Some(DValue::Number(v))
    } else if s == "null" {
        Some(DValue::Null)
    } else if s == "undefined" {
        Some(DValue::Undefined)
    } else {
        Some(DValue::String(s.to_string()))
    }
}

/// Parse an Avm1 command
fn parse_avm1_command(cmd: &str) -> Option<Command> {
    if smatch(cmd, "help").is_some() {
        println!("Ruffle Debugger Help (AVM1)");
        println!();
        println!("() = Short form");
        println!();
        println!("Values:");
        println!("null - Null");
        println!("undefined - Undefined");
        println!("123.4 - Number");
        println!("123 - Int");
        println!("\"Foo\" - String");
        println!();
        println!("Commands:");
        println!("avm1 break - Break execution at the next instruction");
        println!("avm1 breakpoint list - List active breakpoints");
        println!("avm1 breakpoint add \"function_name\" - Break execution when \"function_name\" is called");
        println!("avm1 breakpoint remove \"function_name\" - Remove a breakpoint");
        println!();
        println!("Only available when in a breakpoint:");
        println!("avm1 (si)/step - Execute next instruction");
        println!("avm1 (sr)/step_out - Continue execution, until returning");
        println!();
        println!("avm1 stack show - Show the current state of the stack");
        println!("avm1 stack push <Value> - Push a value onto the stack");
        println!("avm1 stack pop - Remove the top value from the stack");
        println!();
        println!("avm1 registers show - Show the current state of the registers");
        println!();
        println!("avm1 backtrace - Show the call stack");
        println!("avm1 locals - Show the current local variables");
        println!("avm1 globals - Show global variables");
        println!();
        println!("avm1 get <path> - Get the value of the variable at <path>");
        println!("avm1 set <path> <value> - Set the value of the variable at <path> to <value>");
        println!("avm1 props <path> - Get the sub-properties of the variable at <path>");
        println!();
        println!("avm1 continue");
        println!();
        println!("avm1 help - View this message");
    } else if smatch(cmd, "si").is_some() {
        return Some(Command::Avm1StepInto);
    } else if smatch(cmd, "sr").is_some() {
        return Some(Command::Avm1StepOut);
    } else if let Some(path) = smatch(cmd, "set") {
        let mut parts = path.split(' ');
        let path = parts.next().unwrap();
        let value = parts.next().unwrap();

        if let Some(value) = parse_value(value) {
            return Some(Command::Avm1VariableSet {
                path: path.to_string(),
                value,
            });
        } else {
            return None;
        }
    } else if let Some(path) = smatch(cmd, "props") {
        return Some(Command::Avm1SubpropGet {
            path: path.to_string(),
        });
    } else if let Some(path) = smatch(cmd, "get") {
        return Some(Command::Avm1VariableGet {
            path: path.to_string(),
        });
    } else if let Some(bp) = smatch(cmd, "breakpoint") {
        if let Some(name) = smatch(bp, "add") {
            return Some(Command::Avm1FunctionBreak {
                name: name.to_string(),
            });
        } else if let Some(name) = smatch(bp, "remove") {
            return Some(Command::Avm1FunctionBreakDelete {
                name: name.to_string(),
            });
        } else if smatch(bp, "list").is_some() {
            return Some(Command::Avm1BreakpointsGet);
        }
    } else if smatch(cmd, "break").is_some() {
        return Some(Command::Avm1Break);
    } else if smatch(cmd, "step_out").is_some() {
        return Some(Command::Avm1StepOut);
    } else if smatch(cmd, "step").is_some() {
        return Some(Command::Avm1StepInto);
    } else if smatch(cmd, "registers show").is_some() {
        return Some(Command::Avm1Registers);
    } else if smatch(cmd, "backtrace").is_some() {
        return Some(Command::Avm1Backtrace);
    } else if smatch(cmd, "locals").is_some() {
        return Some(Command::Avm1Locals);
    } else if smatch(cmd, "globals").is_some() {
        return Some(Command::Avm1Globals);
    } else if let Some(stack) = smatch(cmd, "stack") {
        if smatch(stack, "show").is_some() {
            return Some(Command::Avm1Stack);
        } else if let Some(arg) = smatch(stack, "push") {
            if let Some(val) = parse_value(arg) {
                return Some(Command::Avm1Push { val });
            }
        } else if smatch(stack, "pop").is_some() {
            return Some(Command::Avm1Pop);
        }
    } else if smatch(cmd, "continue").is_some() {
        return Some(Command::Avm1Continue);
    }

    None
}

/// Parse a player command
fn parse_player_command(cmd: &str) -> Option<Command> {
    if smatch(cmd, "help").is_some() {
        println!("Ruffle Debugger Help (Player)");
        println!();
        println!("Commands:");
        println!("player pause");
        println!("player resume");
        println!();
        println!("player help - View this message");
    } else if smatch(cmd, "pause").is_some() {
        return Some(Command::Pause);
    } else if smatch(cmd, "resume").is_some() {
        return Some(Command::Play);
    }

    None
}

/// Parse a display object command
fn parse_do_command(cmd: &str) -> Option<Command> {
    if smatch(cmd, "help").is_some() {
        println!("Ruffle Debugger Help (Display Object)");
        println!();
        println!("() = Short form");
        println!();
        println!("Commands:");
        println!("do info <path>");
        println!("do children <path>");
        println!("do props <path>");
        println!("do stop <path>");
        println!("do prop get <path> <prop_name>");
        println!("do prop set <path> <prop_name>");
        println!();
        println!("do help - View this message");
    } else if let Some(path) = smatch(cmd, "info") {
        return Some(Command::Info {
            path: path.to_string(),
        });
    } else if let Some(path) = smatch(cmd, "children") {
        return Some(Command::GetChildren {
            path: path.to_string(),
        });
    } else if let Some(path) = smatch(cmd, "props") {
        return Some(Command::GetProps {
            path: path.to_string(),
        });
    } else if let Some(path) = smatch(cmd, "stop") {
        return Some(Command::StopDO {
            path: path.to_string(),
        });
    } else if let Some(prop) = smatch(cmd, "prop") {
        if let Some(args) = smatch(prop, "get") {
            let next_space_or_end = args.chars().position(|c| c == ' ').unwrap_or(args.len());
            let path = &args[..next_space_or_end];
            let name = &args[next_space_or_end..];
            return Some(Command::GetPropValue {
                path: path.to_string(),
                name: name.to_string(),
            });
        } else if let Some(args) = smatch(prop, "set") {
            let mut parts = args.split(' ');
            let path = parts.next().unwrap();
            let name = parts.next().unwrap();
            let value = parts.next().unwrap();
            if let Some(value) = parse_value(value) {
                return Some(Command::SetPropValue {
                    path: path.to_string(),
                    name: name.to_string(),
                    value,
                });
            }
        }
    }
    None
}

/// Parse a command, top level
pub fn parse_command(cmd: &str) -> Option<Command> {
    if let Some(avm1_cmd) = smatch(cmd, "avm1") {
        parse_avm1_command(avm1_cmd)
    } else if let Some(do_cmd) = smatch(cmd, "do") {
        parse_do_command(do_cmd)
    } else if let Some(player_cmd) = smatch(cmd, "player") {
        parse_player_command(player_cmd)
    } else if smatch(cmd, "quit").is_some() {
        std::process::exit(0);
    } else if smatch(cmd, "help").is_some() {
        println!("Ruffle Debugger Help (Top level)");
        println!();
        println!("Commands:");
        println!("avm1 - Query the state of the AVM1 interpreter");
        println!("do - Query a display object");
        println!("player - Control the state of the player");
        println!("help - View this message");
        println!("quit - Exit the debugger");
        println!();
        println!("Try \"<command> help\" for more details");
        None
    } else {
        None
    }
}
