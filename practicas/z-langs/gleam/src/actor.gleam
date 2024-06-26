import gleam/io
import gleam/erlang/process
import gleam/otp/actor
import gleam/list
import gleam/string

type Message {
  Save(content: String)
  Print
}


fn greeter_loop(message: Message, acc) {
    case message {
      Save(content) -> {
        io.println("save " <> content)
        actor.continue(list.append(acc, [content]))
      }
      Print() -> {
        io.println("Print " <> string.inspect(acc))
        actor.continue([])
      }
      _ -> actor.continue(acc)
    }
}

pub fn main() {

    let assert Ok(greeter) = actor.start([], greeter_loop)

    process.send(greeter, Save("hello"))
    process.send(greeter, Save("world"))
    process.send(greeter, Save("mas"))
    process.send(greeter, Save("cosas"))
    process.send(greeter, Print)

    process.send(greeter, Save("despues"))
    process.send(greeter, Print)

    process.sleep_forever()
}