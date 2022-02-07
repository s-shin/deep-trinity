use std::collections::HashSet;
use core::prelude::*;
use core::CellTypeId;
use grid::{Grid, Y};
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass]
pub struct Cell(CellTypeId);

#[pymethods]
impl Cell {
    #[classattr]
    const EMPTY: Cell = Cell(CellTypeId(0));
    #[classattr]
    const ANY: Cell = Cell(CellTypeId(1));
    #[classattr]
    const S: Cell = Cell(CellTypeId(2));
    #[classattr]
    const Z: Cell = Cell(CellTypeId(3));
    #[classattr]
    const L: Cell = Cell(CellTypeId(4));
    #[classattr]
    const J: Cell = Cell(CellTypeId(5));
    #[classattr]
    const I: Cell = Cell(CellTypeId(6));
    #[classattr]
    const T: Cell = Cell(CellTypeId(7));
    #[classattr]
    const O: Cell = Cell(CellTypeId(8));
    #[classattr]
    const GARBAGE: Cell = Cell(CellTypeId(9));

    #[classmethod]
    pub fn from_id(_cls: &PyType, id: u8) -> PyResult<Self> {
        if id >= 10 {
            Err(pyo3::exceptions::PyValueError::new_err("Invalid cell ID."))
        } else {
            Ok(Self(CellTypeId(id)))
        }
    }
    #[classmethod]
    pub fn from_str(cls: &PyType, s: String) -> PyResult<Self> {
        if s.len() != 1 {
            Err(pyo3::exceptions::PyValueError::new_err("Invalid string representation of cell."))
        } else {
            Self::from_id(cls, CellTypeId::from_char(s.chars().next().unwrap()).0)
        }
    }
    #[getter]
    pub fn id(&self) -> PyResult<u8> { Ok(self.0.0) }
    pub fn __str__(&self) -> PyResult<String> {
        Ok(self.0.to_char().to_string())
    }
}

#[derive(Hash, PartialEq, Eq)]
#[pyclass(name = "Placement")]
pub struct PlacementWrapper {
    placement: Placement,
}

#[pymethods]
impl PlacementWrapper {
    #[new]
    pub fn new(orientation: u8, x: i8, y: i8) -> Self {
        let placement = Placement::new(Orientation::new(orientation), (x, y).into());
        Self { placement }
    }
    #[getter]
    pub fn orientation(&self) -> PyResult<u8> { Ok(self.placement.orientation.id()) }
    #[getter]
    pub fn x(&self) -> PyResult<i8> { Ok(self.placement.pos.0) }
    #[getter]
    pub fn y(&self) -> PyResult<i8> { Ok(self.placement.pos.1) }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("({}, {}, {})", self.placement.orientation.id(), self.placement.pos.0, self.placement.pos.1))
    }
}

#[pyclass(name = "Game")]
pub struct GameWrapper {
    game: Game,
}

#[pymethods]
impl GameWrapper {
    #[new]
    fn new() -> Self {
        Self { game: Default::default() }
    }
    pub fn fast_mode(&mut self) -> PyResult<()> {
        self.game.fast_mode();
        Ok(())
    }
    pub fn should_supply_next_pieces(&self) -> PyResult<bool> {
        Ok(self.game.should_supply_next_pieces())
    }
    pub fn supply_next_pieces(&mut self, piece_cell_ids: Vec<u8>) -> PyResult<()> {
        let mut pieces = Vec::with_capacity(piece_cell_ids.len());
        for cell_id in piece_cell_ids.iter() {
            if let Some(p) = CellTypeId(*cell_id).to_piece() {
                pieces.push(p);
            } else {
                return Err(pyo3::exceptions::PyValueError::new_err("Invalid piece ID given."));
            }
        }
        self.game.supply_next_pieces(&pieces);
        Ok(())
    }
    pub fn setup_falling_piece(&mut self, piece_cell_id: Option<u8>) -> PyResult<()> {
        if piece_cell_id.is_none() {
            return self.game.setup_falling_piece(None).map_err(pyo3::exceptions::PyRuntimeError::new_err);
        }
        if let Some(p) = CellTypeId(piece_cell_id.unwrap()).to_piece() {
            self.game.setup_falling_piece(Some(p)).map_err(pyo3::exceptions::PyRuntimeError::new_err)
        } else {
            Err(pyo3::exceptions::PyValueError::new_err("Invalid piece ID given."))
        }
    }
    pub fn drop(&mut self, n: i8) -> PyResult<()> {
        self.game.drop(n).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    pub fn firm_drop(&mut self) -> PyResult<()> {
        self.game.firm_drop().map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    pub fn shift(&mut self, n: i8, to_end: bool) -> PyResult<()> {
        self.game.shift(n, to_end).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    pub fn rotate(&mut self, n: i8) -> PyResult<()> {
        self.game.rotate(n).map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    pub fn lock(&mut self) -> PyResult<bool> {
        self.game.lock().map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    pub fn hold(&mut self) -> PyResult<bool> {
        self.game.hold().map_err(pyo3::exceptions::PyRuntimeError::new_err)
    }
    pub fn get_dst_candidates(&mut self) -> PyResult<HashSet<PlacementWrapper>> {
        match self.game.get_move_decision_helper(None) {
            Ok(helper) => {
                let r = helper.material.dst_candidates.iter()
                    .map(|&placement| PlacementWrapper { placement })
                    .collect::<HashSet<_>>();
                Ok(r)
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e)),
        }
    }
    pub fn set_playfield_with_u64(&mut self, six_rows_x7: Vec<u64>) -> PyResult<()> {
        if six_rows_x7.len() != 7 {
            return Err(pyo3::exceptions::PyValueError::new_err("Invalid length of values."));
        }
        for (i, v) in six_rows_x7.iter().enumerate() {
            self.game.state.playfield.grid.set_rows_with_bits((0, i as Y * 6).into(), 10, *v);
        }
        Ok(())
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.game))
    }
    fn __copy__(&self) -> PyResult<Self> {
        Ok(Self { game: self.game.clone() })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_wrapper() {
        let mut g = GameWrapper::new();
        g.set_playfield_with_u64(vec![4695472149559893610, 6281985146881357, 1119713820219917312, 0, 0, 0, 0]).unwrap();
        println!("{}", g.__str__().unwrap());
    }
}
