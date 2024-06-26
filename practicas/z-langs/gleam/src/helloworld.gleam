import gleam/io
import gleam/erlang/process
import gleam/string

pub fn main() {

    let parent = process.self()
    let subject = process.new_subject()

    io.println("I'm parent PID" <> string.inspect(parent))

    // Spawn an Erlang process (not an operating system one!)
    process.start(fn() {
      io.println("I'm child PID" <> string.inspect(process.self()))
      process.sleep(1000)
      process.send(subject, "hello world")
    }, True)

    io.println("Parent waiting")

    // Block until the message is received
    case process.receive(subject, 10000) {
      Ok(message) -> io.println("Parent got " <> message)
      _ -> io.println("unexpected")
    }

    io.println("Done")

}