use i3ipc::I3Connection;

fn main() {
    // establish a connection to i3 over a unix socket
    let mut connection = I3Connection::connect().unwrap();

    // request and print the i3 version
    println!("{}", connection.get_version().unwrap().human_readable);

    // fullscreen the focused window
    connection.run_command("fullscreen").unwrap();
}
