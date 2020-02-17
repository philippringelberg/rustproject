use std::collections::{HashMap, HashSet};
use runner::common::*;
use std::ops::Div;
use std::env;

fn main() {
    
    // Sadly i did not have the time to use structopt to read in the command line options
    // This solution also works but is far less pretty
    let mut exact_calculation: bool = false;
    if let Some(arg) = env::args().nth(1){
        if arg == "exact".to_string() {
            exact_calculation = true;
        }
    }
    let t1 = Task {
        id: "T1".to_string(),
        prio: 1,
        deadline: 100,
        inter_arrival: 100,
        trace: Trace {
            id: "T1".to_string(),
            start: 0,
            end: 10,
            inner: vec![
                Trace{
                    id: "R1".to_string(),
                    start: 5, 
                    end: 10, 
                    inner:vec![
                        Trace{
                            id: "R2".to_string(),
                            start: 5,
                            end: 10,
                            inner: vec![]
                        }
                    ]
                }
            ],
        },
    };

    let t2 = Task {
        id: "T2".to_string(),
        prio: 2,
        deadline: 200,
        inter_arrival: 200,
        trace: Trace {
            id: "T2".to_string(),
            start: 0,
            end: 30,
            inner: vec![
                Trace {
                    id: "R1".to_string(),
                    start: 10,
                    end: 20,
                    inner: vec![Trace {
                        id: "R2".to_string(),
                        start: 12,
                        end: 16,
                        inner: vec![],
                    }],
                },
                Trace {
                    id: "R1".to_string(),
                    start: 22,
                    end: 28,
                    inner: vec![],
                },
            ],
        },
    };

    let t3 = Task {
        id: "T3".to_string(),
        prio: 3,
        deadline: 50,
        inter_arrival: 50,
        trace: Trace {
            id: "T3".to_string(),
            start: 0,
            end: 30,
            inner: vec![Trace {
                id: "R2".to_string(),
                start: 10,
                end: 20,
                inner: vec![],
            }],
        },
    };
/*
    let t4 = Task {
        id: "T4".to_string(),
        prio: 1, 
        deadline: 200,
        inter_arrival: 200,
        trace: Trace{
            id: "T4".to_string(),
            start: 0,
            end: 15,
            inner: vec![]
        }
    };
*/

    // builds a vector of tasks t1, t2, t3
    let tasks: Tasks = vec![t1, t2, t3];

    let ( ip, tr) = pre_analysis(&tasks);
        // println!("ip: {:?}", ip);
        // println!("tr: {:?}", tr);

    let (at, ct) = own_analysis(&tasks);
       
    // Calculating and displaying the total CPU load
    let ltot = calculate_ltot(&tasks);
    println!("The load factor on the CPU is: {}", ltot);

    if ltot < 1.0 {
        // Blocking times for each task get calculated
        let bt = calculate_blocking_time(&tasks, &tr);
        
        if !exact_calculation {
            let bpt = calculate_busy_period(&tasks);
            // println!("Bpt is: {:?}", bpt);
            let it: Interference = calculate_interference(&tasks , &ip, &ct, &at , &bpt);
                // println!("it {:?}", it);
            // Calculating the Response time 
            let (rt, rt_possible) = calculate_response_time(&tasks, &ct, &bt, &it);

            if rt_possible {
                // Putting all together with this function
                let finaldisplay: FinalDisplay = final_display(&tasks, &rt, &ct, &bt, &it);
                println!("{:?}", finaldisplay);
            }
        }
        else if exact_calculation {
            let (it, rt, rt_possible) = calculate_exact_response_time(&tasks, &ct, &bt);
            
            if rt_possible {
                // Putting all together with this function
                let finaldisplay: FinalDisplay = final_display(&tasks, &rt, &ct, &bt, &it);
                println!("{:?}", finaldisplay);
            }
        }
            
        
    } 
}


fn own_analysis(tasks: &Tasks) -> (InterTimings , Ct) {
    // Introducing my personal helper functions to get the values needed
    // into HashMaps

    // Creating a HashMap for inter_arrival
    let mut at = HashMap::new();
    
    // ct is the WCET of a task
    let mut ct = HashMap::new();

    for t in tasks {
        at.insert(t.id.clone(), t.inter_arrival);
        let c = u32::from(t.trace.end - t.trace.start);
        ct.insert(t.id.clone(), c);
    }
    (at, ct) 
}

fn calculate_ltot(tasks: &Tasks) -> f64 {
    // ltot is the total load factor which is a value >1
    let mut ltot: f64 = 0.0;
    
    // Here is where the total load factor gets calculated
    for t in tasks {
        let c = f64::from(t.trace.end - t.trace.start);
        let a = f64::from(t.inter_arrival);
        let ltot_temp: f64 = c / a;
        if ltot_temp > 1.0 {
            println!("The wcet times of task {:?} does not match 
                the inter arrival time !", t.id.clone());
            
        }
        // println!("Ltot_temp is: {}", ltot_temp);
        ltot += ltot_temp;
    }

    // ltot now gets checked if it is bigger then one, 
    // giving out an Error Message if so 
    if ltot > 1.0 {
        println!("Warning: Your Load factor exceeds CPU limits !");
        
    }
    ltot
}

fn calculate_blocking_time(tasks: &Tasks, tr: &TaskResources) -> BlockingTime {
    /*
    This function consist out of two parts and is a bit messy, but it does its job
    In the first part, all resource used get analysed and cramped into a vector (blocking_vector)
    The helper function fill_blocking_vector iterates through the layers of each task

    In the second part, the Tasks get analysed and according to their used resources the 
    blocking time gets calculated.
    Finally all comes together in a HashMap to be used in other functions as B(t)
    */
    // t_max is a HashMap that cointains the maximum blocking time of each task
    let mut blocking_vector: BlockingVector = Vec::new();
    let mut bt = HashMap::new();

    // part one
    for t in tasks {
        //defining all data for the blocking vector on task base
        let prio: u8 = t.prio;
       
        for i in &t.trace.inner {
            fill_blocking_vector(i, prio, &mut blocking_vector);
        }

        // The blocking vector contains all resources used by tasks, their 
        // blocking time and their priority, this is too much information
        // which will be sorted out later 
        fn fill_blocking_vector(trace: &Trace, prio: u8, blocking_vector: &mut BlockingVector) {
            let resource = trace.id.clone();
            let time = trace.end - trace.start;
                
            let bf = BlockingFiller {
                resource: resource,
                time: time,
                prio: prio,
            };
            blocking_vector.push(bf);
            
            for cs in &trace.inner {
                fill_blocking_vector(cs, prio, blocking_vector);
            }
        }
    }
    // part two
    for t in tasks {
        // Get the needed information for the task looked at
        // which Resources is it using
        let mut retask = HashSet::new();

        // a list that contains all resources used
        let mut a: bool = false;
        let mut btpart: u32 = 0;

        // I should change this into an if statement to only have the case of some
        // Geting the resources for the task out of the Task resources
        match tr.get(&t.id.clone()){
            Some(retemp) => retask= retemp.clone(),
            None => a= false,
        }

        // Iterating over the resources of a task
        for i in retask {
            let resource_name = i.clone();
            let mut tmax: u32 = 0;
            
            // now iterate through the list of all resources to find 
            // the resource looked at 
            for bf in &blocking_vector{
                let re_look     = bf.resource.clone();
                let time_look   = bf.time;
                let prio_look   = bf.prio;
                // sorting out to find l_r and its max blocking time
                if resource_name == re_look && prio_look < t.prio {
                    if time_look > tmax {
                        tmax = time_look;
                    }
                }
            }
            btpart += tmax;             
        }
        bt.insert(t.id.clone(), btpart);
    }    
    bt
}

fn calculate_busy_period(tasks: &Tasks ) -> Bpt  {
    // parameter to change between exact solution and approximation
    // when true, the formula of the recurrence relation gets used

    // 
    let mut bpt = HashMap::new();

        println! ("Busy-time calculation is approximated. To get the exact solution run: cargo run --bin my_task exact");
        for t in tasks {
            bpt.insert(t.id.clone(), t.deadline);
        }
      
    (bpt)
}

fn calculate_interference (tasks: &Tasks, idprio: &IdPrio, ct: &Ct, at: &InterTimings, bpt: &Bpt) -> Interference {
    // Hashmap where all the I(t) are stored
    let mut it = HashMap::new();

    for t in tasks{
        // find out priority of task
        let prio = t.prio;
            
        let bpttask = readin_u32(&t, &bpt);
        let bpttask = f64::from(bpttask);

        //creating a HashSet that contains all tasks with higher prio
        let mut higher_prio = HashMap::new();

        // Partial sum for the calculation of I(t)
        let mut part_sum: u32 = 0;
        let mut part_mult: u32 = 0;
                
        for (task, tprio) in idprio.iter() {
            if task.contains("T") {
                if tprio > &prio {
                    higher_prio.insert(task, tprio);
                        
                    //Here is where the main code is exectued for the calculation
                    // of the time interval
                    let k = 0;
                    if let Some(k) = ct.get(task) {

                        // It is a bit tricky to get the values of at here
                        // Maybe i find a more elegant solution
                        if let Some(l) = at.get(task){
                            let l64 = f64::from(*l);
                            let c = *k;

                            // Division is worked out here
                            let part_div = f64::from(bpttask / l64);
                            let part_div = f64::ceil(part_div);
                            let part_div = part_div as u32;

                            part_mult = c * part_div;
                            part_sum += part_mult;                           
                        }
                        else {
                            println!("No A(t) found !");
                        }
                    }
                    else {
                        println!("No Bp(t) found");
                    }                     
                }
            }   
        }
        // After having iterated through all higher prio tasks
        // The preemption time can now be added into the HashMap
        it.insert(t.id.clone(), part_sum);
    }

    
    it
}

fn calculate_response_time(tasks: &Tasks, ct: &Ct, bt: &BlockingTime, it: &Interference) -> (ResponseTime, bool) {
    
    /* This function takes care of calculating the response 
    time of each task by the use of the formula: 
        R(t) = C(t) + B(t) + I(t)
    */
    let mut rt = HashMap::new();
    let mut rt_possible = true;

    for t in tasks {
        // extracting all the values needed for the calculation   
        let cttask  = readin_u32(&t, &ct);
        let bttask  = readin_u32(&t, &bt);
        let ittask  = readin_u32(&t, &it);
        // Do calculation
        let rttask = cttask + bttask + ittask;
        if rttask > t.deadline {
            println!("The response time is higher than the deadline!");
            rt_possible = false;
        }
        rt.insert(t.id.clone(), rttask);
    }
    (rt, rt_possible)
}

fn calculate_exact_response_time(tasks: &Tasks, ct: &Ct, bt: &BlockingTime) -> (Interference, ResponseTime, bool) {
    println!("Busy-period calculation is exact");
    let mut rt = HashMap::new();
    let mut it = HashMap::new();
    let mut rt_possible = true;

        for t in tasks {
            // initializing the values for Ci and Bi 
            let ctask = readin_u32(&t, &ct);
            let btask = readin_u32(&t, &bt);
            let prio_task = t.prio;
            let mut ittask: u32 = 0;


            let mut r_new: u32 = 0;
            let mut r_check: u32 = 0; // A value of R to check if it changed to the previous one
            let mut r_old: u32 = ctask + btask; // Initialize the first value
            // println!("{} R0 {}, Ct {}, Bt {}",t.id.clone(), r_old, ctask, btask);

            while r_check != r_old { // Iteration until stable
                let mut sum: u32 = 0;
                r_check = r_old;
                
                for i in tasks { 
                    // Iteration over higher prio tasks
                    if prio_task < i.prio {

                        // sadly everything needs to be f64 for the ceiling function
                        // here is where Ri(s-1)/ Dh happens
                        let r_old64 = f64::from(r_old);
                        let dt_higher64 = f64::from(i.deadline);
                        let division = r_old64 / dt_higher64;
                        let ceiling = division.ceil() as u32;
                        
                        
                        // now multiply the ceiling with C(h)
                        let ct_higher = readin_u32(&i, &ct);
                        sum += ceiling * ct_higher;
                    }   
                }
                
                r_new = ctask + btask + sum ;
                r_old = r_new;
                ittask = sum;
            }
            // Warning message for the case Bpt(t) > D(t)
            if r_new > t.deadline {
                println!("The Busy-Time for task {} is too high! Scheduling is not possible",
                     t.id.clone());
                rt_possible = false;
            }
            
            rt.insert(t.id.clone(), r_new);
            it.insert(t.id.clone(), ittask);
        }
    (it, rt, rt_possible)  
}

fn final_display(tasks: &Tasks, rt: &ResponseTime , ct: &Ct, bt: &BlockingTime, it: &Interference ) -> FinalDisplay {
    // starting off with a growable vector with a filling data type
    let mut vec = Vec::new();
      
    for t in tasks {
        // Reading in
        let task = t.id.clone();
        let rttask = readin_u32(&t, &rt);
        let cttask = readin_u32(&t, &ct);
        let bttask = readin_u32(&t, &bt);
        let ittask = readin_u32(&t, &it);
        
        // Filling in
        let vec_filller = TaskAnalysis {
            task: t.id.to_string(),
            rt: rttask,
            ct: cttask,
            bt: bttask,
            it: ittask,
        };
        // Feed the vector
        vec.push(vec_filller);
    }
   vec
}