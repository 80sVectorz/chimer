# Chimer
Chimer is a pure Rust cli timer &amp; stopwatch application.

**Chimer was made as a way of practicing Rust I am in no way experienced in Rust.**
**Thus this program is probably painful to look at.**

Help message:
``` 
Chimer is a pure Rust cli timer & stopwatch application.
----------------------------------------------------------------------------------------------------------
Usage:
    - chimer -t/--timer DURATION "TIMER NAME" | Starts a timer that chimes when the duration has passed.
                                   The duration has a format of H:M:S .
    - chimer -d/--duration "TIMER NAME" | Checks and shows the time left on the given timer until enter is pressed.
    - chimer -l/--list | Shows a list of all running timers.
    - chimer -s/--stopwatch | Starts a stopwatch that stops when any key is pressed.
    
```

