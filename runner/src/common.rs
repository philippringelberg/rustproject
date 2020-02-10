use std::collections::{HashMap, HashSet};
    /*
    I tried to implement StructOpt here but decided against it
    // use structopt::StructOpt;
    // Providing a command line argument to switch between the exact
    // and approximated calculation of the Busy time period
    #[derive(Debug, StructOpt)]
    struct ExCalc {
        // This option can be specified by -e 
        #[structopt(short)]
        exact_calculation: bool,
    }
    */
// common data structures

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub prio: u8,
    pub deadline: u32,
    pub inter_arrival: u32,
    pub trace: Trace,
}

//#[derive(Debug, Clone)]
#[derive(Debug)]
pub struct Trace {
    pub id: String,
    pub start: u32,
    pub end: u32,
    pub inner: Vec<Trace>,
}
// Used for the final display
#[derive(Debug)]
pub struct TaskAnalysis {
    pub task: String,
    pub rt: u32,
    pub ct: u32,
    pub bt: u32,
    pub it: u32,
}

// Type to document the resource blocking
#[derive(Debug)]
pub struct BlockingFiller {
    pub resource: String,
    pub time: u32,
    pub prio: u8,
}


// uselful types

// Our task set
pub type Tasks = Vec<Task>;

// A map from Task/Resource identifiers to priority
pub type IdPrio = HashMap<String, u8>;

// A map from Task identifiers to a set of Resource identifiers
pub type TaskResources = HashMap<String, HashSet<String>>;

// A map from Task with intertimings
pub type InterTimings = HashMap<String, u32>;

// A map from Traces with WCET timings
pub type Ct = HashMap<String, u32>;

// A blocking vector to list which Task is blocking which Resource for how long
pub type BlockingVector = Vec<BlockingFiller>;

// A map of the busy times of each task
pub type Bpt = HashMap<String, u32>;

// A map of the response times of each task
pub type ResponseTime = HashMap<String, u32>;

// A map of the interference to each task
pub type Interference = HashMap<String, u32>;

// A map from Traces with blocking timings
pub type BlockingTime = HashMap<String, u32>;

// A special data type for the final display form of all results
pub type FinalDisplay = Vec<TaskAnalysis>;


// Derives the above maps from a set of tasks
pub fn pre_analysis(tasks: &Tasks) -> (IdPrio, TaskResources) {
    let mut ip = HashMap::new();
    let mut tr: TaskResources = HashMap::new();
    for t in tasks {
        update_prio(t.prio, &t.trace, &mut ip);
        for i in &t.trace.inner {
            update_tr(t.id.clone(), i, &mut tr);
        }
    }
    (ip, tr)
}

// helper functions
fn update_prio(prio: u8, trace: &Trace, hm: &mut IdPrio) {
    if let Some(old_prio) = hm.get(&trace.id) {
        if prio > *old_prio {
            hm.insert(trace.id.clone(), prio);
        }
    } else {
        hm.insert(trace.id.clone(), prio);
    }
    for cs in &trace.inner {
        update_prio(prio, cs, hm);
    }
}

fn update_tr(s: String, trace: &Trace, trmap: &mut TaskResources) {
    if let Some(seen) = trmap.get_mut(&s) {
        seen.insert(trace.id.clone());
    } else {
        let mut hs = HashSet::new();
        hs.insert(trace.id.clone());
        trmap.insert(s.clone(), hs);
    }
    for trace in &trace.inner {
        update_tr(s.clone(), trace, trmap);
    }
}

pub fn readin_u32(task: &Task, hin: &HashMap<String,u32>) -> u32 {
    let mut out: u32 = 0;
    if let Some(value) = hin.get(&task.id) {
        out = *value;
    }
    out
}
