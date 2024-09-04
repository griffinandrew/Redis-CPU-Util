use procfs::process::{Process};
use sysinfo::{System, Pid};
use std::{thread, time::Duration, fs::OpenOptions, io::Write};
use clock_ticks::{precise_time_s};


#[derive(Default, Debug, Clone)]
struct Stats {
    cpu_total_time: Vec<f64>,
    sys_time: Vec<f64>,
}

impl Stats {
    
    fn print_stats(&self) {
        println!("CPU Total Time: {:?}", self.cpu_total_time);
        println!("System Time: {:?}", self.sys_time);
    }

    //new method of getting data
    fn data_record(&self, results_file: String) {
        
        let mut file = OpenOptions::new()
            .write(true)
            .append(true) // open for appending
            .create(true) // to create the file create file if DNE 
            .open(results_file.clone())
            .expect("Error opening file");
 

        //get exp run time
        //there should only be one 
        let exp_time = self.sys_time.get(0).expect("Could not get the last element of sys time");

        //get exp run time
        let cpu_start_time = self.cpu_total_time.get(0).expect("Could not get the first element of cpu total time");
        let cpu_end_time = self.cpu_total_time.get(1).expect("Could not get the last element of cpu total time");
        let cpu_duration = cpu_end_time - cpu_start_time;
        let cpu_util = (cpu_duration / *exp_time) * 100.0;

        let res  = writeln!(file, "CPU: {}", cpu_util);
        if res.is_err() {
            eprintln!("Error writing to file {}", results_file);
        }
        println!("CPU Utilization: {}", cpu_util);
        //end exp runtime

    }
}


fn get_redis_cpu(res_file: String, mut stats: Stats, run_length: i32) {
    let sys_1 = System::new_all();
    //needed to do this to to cast to i32 to track the proc info
    let mut redis_pid:Pid = 0.into();
    let mut i32_pid_redis = 0;

    for (pid, process) in sys_1.processes(){
        if process.name() == "redis-server" {
            redis_pid = *pid;
            i32_pid_redis = redis_pid.as_u32() as i32;
            break;
        }
    }


    let redis = Process::new(i32_pid_redis).expect("Could not get the process");
    let ticks_per_second = procfs::ticks_per_second() as f64;
    let start_ticks = precise_time_s(); 

    //end_run is sample size
    let end_run = run_length;
    for i in 0..end_run {

        //at init get all stats
        if i == 0 {
            let init_stat = redis.stat().expect("Could not get the redis time stat");
            //the sum of all cpu times in ticks
            let mut total_time = init_stat.stime as f64 + init_stat.utime as f64 + init_stat.cutime as f64 + init_stat.cstime as f64;
            //divide by ticks per second to get time in seconds
            total_time = total_time / ticks_per_second;
            stats.cpu_total_time.push(total_time as f64);
        }
        //at end of run get all stats
        if i == end_run - 1 {
            //cpu stats
            let init_stat = redis.stat().expect("Could not get the redis time stat");
            //the sum of all cpu times in ticks
            let mut total_time = init_stat.stime as f64 + init_stat.utime as f64 + init_stat.cutime as f64 + init_stat.cstime as f64;
            //divide by ticks per second to get time in seconds
            total_time = total_time / ticks_per_second;
            stats.cpu_total_time.push(total_time as f64);
            //end cpu stats

            //do the time interval so can be sure calcing util of cpu properly
            let curr_time = precise_time_s();
            let time_s = curr_time - start_ticks;
            stats.sys_time.push(time_s);
            //time over
        }
        //always sleep for 1 second (perhaps use a precise timer to be even more accurate?)
        thread::sleep(Duration::from_secs(1));
    } 
    stats.data_record(res_file);
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
        if args.len() != 2 {
        eprintln!("Error: Usage: enter the length of sampling period in seconds");
        std::process::exit(1);
    }
    let test_length = args[1].parse::<i32>().unwrap();
    let cpu_stats = Stats::default(); 
    let file_name = "redis_cpu_stats.txt".to_string();
    get_redis_cpu(file_name, cpu_stats, test_length);
}
