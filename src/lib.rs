use std::collections::HashMap;
use ndarray::{stack, Array, Array2, Axis, s};
use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use std::fmt;

#[derive(Debug, Deserialize, Serialize)]
pub enum Policy{
    Short(String),
    Long((String, String))
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let d = match self {
            Policy::Short(d) => d,
            Policy::Long((d, _)) => d,
        };
        write!(f, "{}", d)
    }
}

pub fn match_by_string(policy: &Policy, id: &str) -> bool {
    match policy {
        Policy::Short(desc) => desc == id,
        Policy::Long((title, _)) =>  title == id,
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Setting {
    pub title: Option<String>,
    pub voters: Vec<String>,
    pub policies: Vec<Policy>,
    pub votes: HashMap<String, HashMap<String, f64>>,
}

impl Setting {
    pub fn add_voter(&mut self, p: &str) {
        self.voters.push(p.to_string());
    }
    pub fn delete_voter(&mut self, p: &str) -> Option<usize> {
        if let Some(index) = self.voters.iter().position(|v| v == p) {
                self.voters.remove(index);
                return Some(index);
        }
        None
    }

    pub fn add_policy(&mut self, p: Policy) {
        self.policies.push(p);
    }

    pub fn delete_policy(&mut self, p: &str) -> Option<usize> {
        if let Some(index) = self.policies.iter().position(|v| match_by_string(v, p)) {
                self.policies.remove(index);
                return Some(index);
        }
        None
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PollResult {
    pub votes: HashMap<String, f64>,
    pub influence: HashMap<String, f64>
}

const ITERATION: u32 = 1000;

pub fn create_matrix(settings: &Setting) -> Array2<f64> {
    let voters = &settings.voters;
    // TODO: check for duplicates
    let policies = &settings.policies;

    let elements_num = voters.len() + policies.len();

    let mut m = Array::zeros((elements_num, voters.len()));

    let mut i = 0;
    for v in voters {
        let vote: &HashMap<String, f64>= settings.votes.get(v).unwrap();
        // TODO: check if there is no one missed out
        for (key, val) in vote.iter() {
            let id = match voters.iter().position(|k| k == key) {
                Some(n) => Some(n),
                None => match &policies.iter().position(|k| match_by_string(k, key)) {
                    Some(n) => Some(n + voters.len()),
                    None => None,
                }
            };

            if let Some(index) = id {
                m[[index, i]] = val.to_owned();
            }
        }

        i += 1;
    }

    let vp :Array2::<f64> = Array::zeros((policies.len(), voters.len()));
    let pp :Array2::<f64> = Array::eye(policies.len());

    let leftpart = stack![Axis(1), vp, pp];
   
    let initial_matrix = stack![Axis(1), m, leftpart.t()];

    initial_matrix
}

pub fn calculate(m: Array2::<f64>, num_voters: usize) -> (Vec<f64>, Vec<f64>){
    let square = m.shape()[0];
    let mut a = Array::eye(square);
    let mut sum = Array::eye(square);

    for _i in 0..ITERATION{
        a = a.dot(&m);
        sum += &a;
    }

    let sum = sum.slice(s![..num_voters, ..num_voters]);
    let sum_row = sum.sum_axis(Axis(1));
    let a = a.slice(s![.., 0..3]);
    let vote_results = a.sum_axis(Axis(1));
    let vote_results = vote_results.slice(s![num_voters..]).to_vec();
    let voters_influence = (sum_row / sum.diag()).to_vec();

    (vote_results, voters_influence)
}

pub fn poll_result(voters: &[String], policies: &[Policy], result: (Vec<f64>, Vec<f64>)) -> PollResult {

    let mut votes_r = HashMap::new();

    let mut influences_r = HashMap::new();

    let (votes, influence) = result;

    for (i, p) in policies.iter().enumerate() {
        let d = format!("{}", p);
        votes_r.insert(d.to_owned(), votes.get(i).unwrap().to_owned());
    }

    for (i, inf) in voters.iter().enumerate() {
        influences_r.insert(inf.to_owned(), influence.get(i).unwrap().to_owned());
    }

    PollResult {
        votes: votes_r,
        influence: influences_r,
    }
}
