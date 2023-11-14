using Distributed

println("before addprocs")
addprocs(4)
println("after addprocs")

@everywhere function count_file_words(file)
    println("PID: ", getpid())
    words = 0
    for line in eachline(file)
        words += length(split(line, " "))
    end
    return words
end

basedir = (@__DIR__) * "/../../2-forkjoin/data"
files = readdir(basedir)

s = @distributed (+) for i = 0:(length(files) * 5)
    count_file_words(basedir * "/" * files[(i % length(files)) + 1])
end

println(s)