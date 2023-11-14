parent = self()

IO.puts("I'm parent PID #{inspect parent}")

# Spawn an Elixir process (not an operating system one!)
spawn_link(fn ->
  IO.puts("I'm child PID #{inspect self()}")
  send(parent, {:this_is_an_atom_to_identify_the_message_type, "hello world"})
end)

IO.puts("Parent waiting")
# Block until the message is received
receive do
  {:this_is_an_atom_to_identify_the_message_type, contents} -> IO.puts(contents)
end

IO.puts("Done")
