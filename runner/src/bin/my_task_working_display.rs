use std::collections::{HashMap, HashSet};
use runner::common::*;
use std::ops::Div;

// Sadly I did not find a way to easily outsource the task generation
// I could try to do it in the source directory
fn main() {
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
    println!("ip: {:?}", ip);
    println!("tr: {:?}", tr);

    let (inter_timing, ct, bpt , ltot, t_block) = own_analysis(&tasks);
    /* debug only
    println!("inter timings {:?}, WCETS are  {:?},
    the deadlines are {:?}, 
    Total load factor is: {} Blocking times are: {:?}", 
    inter_timing, ct, bpt, ltot, t_block);
    */ 

    let it: Interference = calculate_interference(&tasks , &ip, &ct, inter_timing , &bpt);

    // restructure the naming scheme, this is only singe Strings and 
    // integers
    let finaldisplay: FinalDisplay = final_display(&tasks, &tr, &ct, &bpt, &it);
    println!("{:?}", finaldisplay);

}










// Introducing my personal helper functions to get the values needed
// into HashMaps
fn own_analysis(tasks: &Tasks) -> (At , Ct , Bpt , f64, TBlock) {
    //Creating a HashMap for inter_arrival
    let mut at = HashMap::new();
    
    // ltot is the total load factor which is a value >1
    let mut ltot: f64 = 0.0;

    //ct is the WCET of a task
    let mut ct = HashMap::new();

    // bp is the deadline of each task
    let mut bpt = HashMap::new();

    for t in tasks {
        at.insert(t.id.clone(), t.inter_arrival);
        // Maybe further loops are needed to find the right trace
        // within a task
        let c = u32::from(t.trace.end - t.trace.start);
        ct.insert(t.id.clone(), c);
        bpt.insert(t.id.clone(), t.deadline);
    }
      
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

    // t_max is a HashMap that cointains the maximum blocking time of each
    // Task, it is not the required function as in the Exam.md so far
    let mut t_max = HashMap::new();

    for t in tasks {
        let mut t_help: u32 = 0;
        for trace in &t.trace.inner {
            let t_temp = u32::from(trace.end - trace.start);
            // println!("t_help is: {}", t_temp);
            if t_temp > t_help {
                t_help = t_temp;
            }
        }
        t_max.insert(t.id.clone(), t_help);
    }
    (at, ct, bpt, ltot, t_max) 
}


/*  Implement a function that takes a Task and returns the 
    corresponding preemption time.
    Assumptions: 
        Bp(t) = D(t) = task.deadline

    To Do: creating a HashMap with higher priority tasks, by 
*/
// Assign their Deadlines into a HashMap to have Bp(t)

fn calculate_interference (tasks: &Tasks, idprio: &IdPrio, ct: &Ct, at: At, bpt: &Bpt) -> Interference {

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
    // debug only
    // println!("The final result for I(t) is {:?}", it);
}
fn final_display(tasks: &Tasks, tr: &TaskResources, ct: &Ct, bpt: &Bpt, it: &Interference ) -> FinalDisplay {
    // starting off with a growable vector with a filling data type
    let mut vec = Vec::new();
    // probably outsource this to the common.rs
    

    for t in tasks {
        // need a bunch of variables to read out the values
        // from the HashMaps given
        let task = t.id.clone();
        let mut rt = HashSet::new();
        let mut a: bool = false;
        match tr.get(&task) {
            Some(rtemp) => rt = rtemp.clone(),
            None => a = rt.insert(String::from("No resources required")) 
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
        
        let vec_filller = VecFiller {
            task: t.id.to_string(),
            rt: rt,
            ct: ctask,
            bpt: bpttask,
            it: ittask,
        };

        vec.push(vec_filller);
    }

   vec
}
