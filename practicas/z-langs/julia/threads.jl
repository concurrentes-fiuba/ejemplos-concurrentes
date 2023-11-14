function isprime(x::Int)
    if x < 2
        return false
    end
    u = isqrt(x) # integer square root
    for n in 2:u
        if x % n == 0
            return false
        end
    end
    return true
end

# set with -t xx or -t auto
println(Threads.threadpoolsize())

@time for i = 1000:9999999
	isprime(i)
end;

@time Threads.@threads for i = 1000:9999999
	isprime(i)
end;
