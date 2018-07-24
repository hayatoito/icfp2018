use std;
use std::collections::HashSet;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use super::prelude::*;

pub struct Model {
    pub id: ModelId,
    pub r: usize,
    pub bytes: Vec<u8>,
    pub targets: HashSet<Cord>,
}

#[derive(Copy, Clone, Debug)]
pub enum ModelId {
    Assemble(Option<usize>),
    Disassemble(Option<usize>),
}

impl ModelId {
    pub fn name(&self) -> String {
        use self::ModelId::*;
        match self {
            Assemble(Some(id)) => format!("FA{:03}", id),
            Assemble(None) => "Assemple(None)".to_string(),
            Disassemble(Some(id)) => format!("FD{:03}", id),
            Disassemble(None) => "Disassemple(Unknown)".to_string(),
        }
    }

    pub fn filename(&self) -> String {
        use self::ModelId::*;
        match self {
            Assemble(Some(id)) => format!("FA{:03}_tgt.mdl", id),
            Assemble(None) => unreachable!(),
            Disassemble(Some(id)) => format!("FD{:03}_src.mdl", id),
            Disassemble(None) => unreachable!(),
        }
    }
}

impl Model {
    pub fn read_contest_model(id: ModelId) -> Result<Model> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(format!("contest/model/{}", id.filename()));
        Model::read(id, path)
    }

    pub fn read(id: ModelId, path: impl AsRef<Path>) -> Result<Model> {
        let path = path.as_ref();
        debug!("read: {}", path.display());
        let mut f = std::fs::File::open(path)?;

        let mut head = [0; 1];
        f.read_exact(&mut head)?;
        let r = head[0] as usize;

        let mut bytes = vec![];
        let size = f.read_to_end(&mut bytes)?;
        debug!("bytes: size: {}", size);

        let mut targets = HashSet::new();
        for x in 0..r {
            for y in 0..r {
                for z in 0..r {
                    let index = x * r * r + y * r + z;
                    let bi = index / 8;
                    let br = index % 8;
                    if ((bytes[bi] >> br) & 0b_1) != 0 {
                        targets.insert(Cord::new(x as i32, y as i32, z as i32));
                    }
                }
            }
        }

        Ok(Model {
            id,
            r,
            bytes,
            targets,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn model_test() {
        let model = Model::read_contest_model(ModelId::Assemble(Some(1))).unwrap();
        assert_eq!(model.r, 20);
        assert_eq!(model.bytes.len(), 20 * 20 * 20 / 8);
        assert_eq!(
            model
                .bytes
                .iter()
                .map(|byte| byte.count_ones())
                .sum::<u32>(),
            511
        );
        assert_eq!(model.targets.len(), 511);
    }

}
