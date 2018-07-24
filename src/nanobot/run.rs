use serde_json;

use std;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::ai::*;
use super::model::*;
use super::prelude::*;
use super::system::*;
use super::trace::*;
use rayon::prelude::*;

pub struct RunResult {
    pub ai: Ai,
    pub energy: Option<i64>,
    pub system: System,
}

impl RunResult {
    pub fn trace(&self) -> Trace {
        Trace {
            cmds: self.system.records.clone(),
        }
    }

    pub fn model_name(&self) -> String {
        self.system.model_id.name()
    }

    pub fn write_trace(&self) -> Result<()> {
        self.trace().write_to_trace_dir(&format!(
            "{}-{:?}-{:?}.nbt",
            self.model_name(),
            self.ai,
            self.energy
        ))
    }
}

pub fn run(
    bots: Option<usize>,
    src: Option<String>,
    target: Option<String>,
    output: Option<String>,
) -> Result<()> {
    let model = if let Some(src) = src {
        Model::read(ModelId::Disassemble(None), src)?
    } else if let Some(target) = target {
        Model::read(ModelId::Assemble(None), target)?
    } else {
        unreachable!()
    };

    let run_result = solve(&model, Ai::Many(bots.unwrap_or(2)))?;
    if let Some(energy) = run_result.energy.as_ref() {
        println!("{}", energy);
        info!("trace: len: {}", run_result.system.records.len())
    } else {
        eprintln!("failed");
        info!("trace: len: {}", run_result.system.records.len())
    }
    if let Some(output) = output {
        info!("Writing trace to: {}", output);
        run_result.trace().write_to(output)?;
    }
    Ok(())
}

pub fn solve(model: &Model, ai: Ai) -> Result<RunResult> {
    let model_id = model.id;
    let mut system = System::new(model);
    let result = match ai {
        Ai::Many(bots) => Many::new(bots).solve(&mut system),
    };
    match result {
        Ok(_) => Ok(RunResult {
            energy: Some(system.energy),
            ai,
            system,
        }),
        Err(_) => {
            warn!("Failed to solve: model: {}, ai: {:?}", model_id.name(), ai);
            Ok(RunResult {
                energy: None,
                ai,
                system,
            })
        }
    }
}

// contest/submit/submit.json
#[derive(Serialize, Deserialize, Debug)]
struct Submit {
    best_scores: HashMap<String, BestScore>,
}

#[derive(Serialize, Deserialize, Debug)]
struct BestScore {
    ai: String,
    energy: i64,
}

impl Submit {
    fn read_submit_json() -> Result<HashMap<String, BestScore>> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("contest/submit/submit.json");
        if !path.exists() {
            Ok(Default::default())
        } else {
            Ok(serde_json::from_str(&std::fs::read_to_string(path)?).unwrap())
        }
    }

    fn read() -> Result<Submit> {
        Ok(Submit {
            best_scores: Submit::read_submit_json()?,
        })
    }

    fn write(&self) -> Result<()> {
        debug!("Writing submit");
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("contest/submit/submit.json");
        std::fs::write(path, serde_json::to_string(&self.best_scores)?)?;
        Ok(())
    }

    fn is_best(&self, problem_name: &str, energy: i64) -> bool {
        if let Some(best_score) = self.best_scores.get(problem_name) {
            energy < best_score.energy
        } else {
            true
        }
    }

    fn write_best_trace_if(&mut self, run_result: &RunResult) -> Result<()> {
        assert!(run_result.energy.is_some());
        let energy = run_result.energy.unwrap();
        if self.is_best(&run_result.model_name(), energy) {
            info!(
                "Found the best score: model: {}, ai: {:?}, energy: {}",
                run_result.model_name(),
                run_result.ai,
                energy
            );
            self.best_scores.insert(
                run_result.model_name().to_string(),
                BestScore {
                    ai: format!("{:?}", run_result.ai),
                    energy,
                },
            );
            run_result
                .trace()
                .write_to_submit_dir(&run_result.model_name())?;
            self.write()?;
        }
        Ok(())
    }
}

fn ci_run_bots(model_id: ModelId, ais: &[Ai], submit: &Arc<Mutex<Submit>>) {
    ais.par_iter().for_each(|ai| {
        if let Ok(run_result) = solve(&Model::read_contest_model(model_id).unwrap(), *ai) {
            run_result.write_trace().unwrap();
            if run_result.energy.is_some() {
                let mut submit = submit.lock().unwrap();
                submit.write_best_trace_if(&run_result).unwrap();
            }
        }
    });
}

pub fn ci() -> Result<()> {
    let mut model_id_list = vec![];
    for i in 1..=180 {
        model_id_list.push(ModelId::Assemble(Some(i)));
        model_id_list.push(ModelId::Disassemble(Some(i)));
    }
    let ais = vec![2, 3, 4, 6, 8, 12, 20]
        .into_iter()
        .map(Ai::Many)
        .collect::<Vec<_>>();

    let submit = Arc::new(Mutex::new(Submit::read()?));
    model_id_list
        .par_iter()
        .for_each(|model_id| ci_run_bots(*model_id, &ais, &submit));
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn assemble_test() {
        let expected_energy = [(1, 955, 11522830)];
        for (id, cmds, energy) in &expected_energy {
            let model = Model::read_contest_model(ModelId::Assemble(Some(*id as usize))).unwrap();
            let run_result = solve(&model, Ai::Many(2)).unwrap();
            assert_eq!(run_result.energy.unwrap(), *energy);
            assert_eq!(run_result.trace().cmds.len(), *cmds);
        }
    }

    #[test]
    fn disassemble_test() {
        let expected_energy = [(1, 913, 11029332)];
        for (id, cmds, energy) in &expected_energy {
            let model =
                Model::read_contest_model(ModelId::Disassemble(Some(*id as usize))).unwrap();
            let run_result = solve(&model, Ai::Many(2)).unwrap();
            assert_eq!(run_result.energy.unwrap(), *energy);
            assert_eq!(run_result.trace().cmds.len(), *cmds);
        }
    }

}
