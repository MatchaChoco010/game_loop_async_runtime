pub trait World: 'static {
    type Command;
    fn process_command(&mut self, cmd: Self::Command);
}
