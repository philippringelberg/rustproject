use std::collections::{HashMap, HashSet};
use runner::common::*;
use std::ops::Div;
use std::env;



// Sadly I did not find a way to easily outsource the task generation
// I could try to do it in the source directory
fn main() {
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
            inner: vec![],
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

    // builds a vector of tasks t1, t2, t3
    let tasks: Tasks = vec![t1, t2, t3];

    let ( ip, tr) = pre_analysis(&tasks);
        // println!("ip: {:?}", ip);
        // println!("tr: {:?}", tr);

    let (at, ct, dt ) = own_analysis(&tasks);
       
    // Calculating and displaying the total CPU load
    let ltot = calculate_ltot(&tasks);
    println!("The load factor on the CPU is: {}", ltot);

    if ltot < 1.0 {
        // Blocking times for each task get calculated
        let bt = calculate_blocking_time(&tasks, &tr);
        // println!("The blocking time of the tasks is: {:?}", bt);

        let (bpt, bpt_possible) = calculate_busy_period(&tasks, &ct, &bt, &ip, exact_calculation);
    
        let it: Interference = calculate_interference(&tasks , &ip, &ct, &at , &bt);

        // Calculating the Response time 
        let rt: ResponseTime = calculate_response_time(&tasks, &ct, &bt, &it);
        // println!("The response time of each task is: {:?}", rt);
            
        if bpt_possible {
            // Putting all together with this function
            let finaldisplay: FinalDisplay = final_display(&tasks, &rt, &ct, &bpt, &it);
            println!("{:?}", finaldisplay);
        }
    } 
}





// Introducing my personal helper functions to get the values needed
// into HashMaps
fn own_analysis(tasks: &Tasks) -> (InterTimings , Ct , Dt) {
    //Creating a HashMap for inter_arrival
    let mut at = HashMap::new();
    
    //ct is the WCET of a task
    let mut ct = HashMap::new();

    // bp is the deadline of each task
    let mut dt = HashMap::new();

    for t in tasks {
        at.insert(t.id.clone(), t.inter_arrival);
        // Maybe further loops are needed to find the right trace
        // within a task
        let c = u32::from(t.trace.end - t.trace.start);
        ct.insert(t.id.clone(), c);
        dt.insert(t.id.clone(), t.deadline);
    }
    (at, ct, dt) 
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
        ltot = ltot + ltot_temp;
    }

    // ltot now gets checked if it is bigger then one, 
    // giving out an Error Message if so 
    if ltot > 1.0 {
        println!("Warning: Your Load factor exceeds CPU limits !");
        
    }
    ltot
}

fn calculate_blocking_time(tasks: &Tasks, tr: &TaskResources) -> TBlock {
    /*
    This function consist out of two parts and is a bit messy, but it does its job
    In the first part, all resource uses get analysed and cramped into a vector (blocking_vector)
    The helper function fill_blocking_vector iterates through the layers of each task

    In the second part, the Tasks get analysed and according to their used resources the 
    blocking time gets calculated.
    Finally all comes together in a HashMap to be used in other functions as B(t)
    */
    // t_max is a HashMap that cointains the maximum blocking time of each task
    let mut blocking_vector: BlockingVector = Vec::new();
    let mut bt = HashMap::new();

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
                // task: 
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
                let re_look = bf.resource.clone();
                let time_look = bf.time;
                let prio_look = bf.prio;
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

fn calculate_busy_period(tasks: &Tasks, ct: &Ct, bt: &TBlock, ip: &IdPrio, is_exact: bool ) -> (Bpt, bool)  {
    // parameter to change between exact solution and approximation
    // when true, the formula of the recurrence relation gets used

    // 
    let mut bpt = HashMap::new();
    let mut bpt_possible = true;

    if !is_exact {
        println! ("Busy-time calculation is approximated");
        for t in tasks {
            bpt.insert(t.id.clone(), t.deadline);
        }
    }
    


    if is_exact {
        println!("Busy-period calculation is exact");

        for t in tasks {
            // initializing the values for Ci and Bi 
            let ctask = readin_u32(&t, &ct);
            let btask = readin_u32(&t, &bt);
            let prio_task = t.prio;


            let mut bpt_new: u32 = t.deadline;
            let bpt_old: u32 = ctask + btask;
            let mut sum: u32 = 0;
            for j in tasks {
                // this creates the sum that gets added into the iteration
                for i in tasks {
                    // Iteration over higher prio tasks
                    if prio_task < i.prio {

                        // sadly everything needs to be f64 for the ceiling function
                        // here is where Ri(s-1)/ Dh happens
                        let bpt_old64 = f64::from(bpt_old);
                        let dt_higher64 = f64::from(i.deadline);
                        let ceiling64 = f64::ceil(bpt_old64 / dt_higher64 );
                        let ceiling = ceiling64 as u32;
                        
                        // now multiply the ceiling with C(h)
                        let ct_higher = readin_u32(&i, &ct);
                        let part_sum = ceiling * ct_higher;
                        sum += part_sum;
                    }   
                }

                bpt_new = ctask + btask + sum;
            }

            if bpt_new > t.deadline {
                println!("The Busy-Time for task {} is too high! Scheduling is not possible",
                     t.id.clone());
                bpt_possible = false;
            }
            bpt.insert(t.id.clone(), bpt_new);            
        }        
    }  
    (bpt, bpt_possible)
}

fn calculate_interference (tasks: &Tasks, idprio: &IdPrio, ct: &Ct, at: &InterTimings, bpt: &Bpt) -> Interference {
    /*  Implement a function that takes a Task and returns the 
        corresponding preemption time.
        Assumptions: 
            Bp(t) = D(t) = task.deadline
    */

    // Hashmap where all the I(t) are stored
    let mut it = HashMap::new();


    for t in tasks{
        // find out priority of task
        let prio = t.prio;
            // #println!("Task looked at is: {:?}", t.id.clone());

        // Watch out, only the assumption of Bpt works here
        let bpt_task = f64::from(t.deadline);

        //creating a HashSet that contains all tasks with higher prio
        let mut higher_prio = HashMap::new();

        // Partial sum for the calculation of I(t)
        let mut part_sum: f64 = 0.0;
        let mut part_mult: f64 = 0.0;
        
        
        for (task, tprio) in idprio.iter() {
            if task.contains("T") {
                if tprio > &prio {
                    higher_prio.insert(task, tprio);
                        // #println!("Task with higher prio is : {:?}", task);

                    //Here is where the main code is exectued for the calculation
                    // of the time interval
                    let k = 0;
                    if let Some(k) = ct.get(task) {

                        // It is a bit tricky to get the values of at here
                        // Maybe i find a more elegant solution
                        if let Some(l) = at.get(task){
                                // #println!("At for {} is {}", task, l);
                                // #println!("Ct for {} is: {:?}",task,  k);
                            let l64 = f64::from(*l);
                            let c64 = f64::from(*k);

                            // Lets get the division created here
                            let part_div = f64::from(bpt_task / l64);
                            let part_div = f64::ceil(part_div);
                                // #println!("the part_div is  {}", part_div);

                            // The multiplication takes part here
                            part_mult = c64 * part_div;
                                // #println!("The part mult is: {}", part_mult);
                            
                            // Still not working over multiple values
                            // there needs to be an iteration over part_mul
                            part_sum += part_mult;
                            it.insert(t.id.clone(), part_sum);
                        
                        }
                        else {
                            println!("No At found !");
                        }
                    }
                    else {
                        println!("No Bpt found");
                    }                     
                }
            }   
        }
    }

    it
}

fn calculate_response_time(tasks: &Tasks, ct: &Ct, bt: &TBlock, it: &Interference) -> ResponseTime {
    
    /* This function takes care of calculating the response 
    time of each task by the use of the formula: 
        R(t) = C(t) + B(t) + I(t)
    */
    let mut rt = HashMap::new();

    for t in tasks {
        // extracting all the values needed for the calculation
        let mut cttask: u32 = 0;
        match ct.get(&t.id.clone()) {
            Some(cttemp) => cttask = *cttemp,
            None => println!("No correct WCET found"),
        }
        let cttask = f64::from(cttask);

        let mut bpttask: u32 = 0;
        match bt.get(&t.id.clone()) {
            Some(bpttemp) => bpttask = *bpttemp,
            None => println!("No blocking time given"),
        }
        let bpttask = f64::from(bpttask);

        let mut ittask: f64 = 0.0;
        match it.get(&t.id.clone()) {
            Some(ittemp) => ittask = *ittemp,
            None => ittask = 0.0,
        }
        let rttask = cttask + bpttask + ittask;
        rt.insert(t.id.clone(), rttask);
    }
    rt
}

fn final_display(tasks: &Tasks, rt: &ResponseTime , ct: &Ct, bpt: &Bpt, it: &Interference ) -> FinalDisplay {
    // starting off with a growable vector with a filling data type
    let mut vec = Vec::new();
    // probably outsource this to the common.rs
    

    for t in tasks {
        // need a bunch of variables to read out the values
        // from the HashMaps given
        let task = t.id.clone();
        
        // completely wrong formula for rt
        /* keep this as comment, since it is for reading out a HashSet
        let mut rt = HashSet::new();
        let mut a: bool = false;
        match tr.get(&task) {
            Some(rtemp) => rt = rtemp.clone(),
            None => a = rt.insert(String::from("No resources required")) 
        }
        */
        let mut rttask: f64 = 0.0;
        match rt.get(&task) {
            Some(rttemp) => rttask = *rttemp,
            None => println!("There has been an error calculating the response time"),
        }
        
        let mut ctask:u32 = 0;
        match ct.get(&task) {
            Some(cttemp) => ctask = *cttemp,
            None => println!("No correct WCET available")
        }

        let mut bpttask: u32 = 0;
        match bpt.get(&task) {
            Some(bpttemp) => bpttask = *bpttemp,
            None => println!("No busy period available")
        }
        
        let mut ittask: f64 = 0.0;
        match it.get(&task) {
            Some(ittemp) => ittask = *ittemp,
            None => println!("Task {} is not preempted", task),
        }
        
        let vec_filller = TaskAnalysis {
            task: t.id.to_string(),
            rt: rttask,
            ct: ctask,
            bpt: bpttask,
            it: ittask,
        };

        vec.push(vec_filller);
    }

   vec
}
