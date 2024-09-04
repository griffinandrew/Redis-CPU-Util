# Redis CPU Util Tracker
This makes use of the sysinfo crate to search the system process list to locate the pid of the redis server. This pid is then passed to profcs to monitor redis's cpu usage over a certain time duration in seconds specified by the user. The utilization is then calculated from the busy time of the server divided by the experiement sample duration. This CPU utilization is then written to a file and the terminal.

## Example Usage 

cargo run 11 

cargo run 100
