# Chimer
Chimer is a pure Rust cli timer &amp; stopwatch application.

**Chimer was made as a way of practicing Rust I am in no way experienced in Rust.**
**Thus this program is probably painful to look at.**

## Help message:
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

## How to install:
Clone the repo and build it using cargo then copy the executable and `alarmSound.mp3` to a new directory in my case:`/bin/chimerBin`. Create a symbolic link to the executable in /bin/ outside of the program folder. Make sure the program files folder has writing and reading enabled most likely using `chmod`. Using chimer requires this the exe will panic if this is not done no exception handeling has been implemented for permision related exceptions. If you do get an exception then please remove the `timer.yaml` file that might have been created and start a new timer as a test.
