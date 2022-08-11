use std::collections::{HashSet, VecDeque};
use core::prelude::*;
use core::helper::MoveDecisionResource;
use grid::{Grid, Y, Cell as _};
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass(name = "Cell")]
pub struct CellWrapper(Cell);

#[pymethods]
impl CellWrapper {
    #[classattr]
    const EMPTY: CellWrapper = Self(Cell::Empty);
    #[classattr]
    const ANY: CellWrapper = Self(Cell::Any);
    #[classattr]
    const S: CellWrapper = Self(Cell::S);
    #[classattr]
    const Z: CellWrapper = Self(Cell::Z);
    #[classattr]
    const L: CellWrapper = Self(Cell::L);
    #[classattr]
    const J: CellWrapper = Self(Cell::J);
    #[classattr]
    const I: CellWrapper = Self(Cell::I);
    #[classattr]
    const T: CellWrapper = Self(Cell::T);
    #[classattr]
    const O: CellWrapper = Self(Cell::O);
    #[classattr]
    const GARBAGE: CellWrapper = Self(Cell::Garbage);

    #[classmethod]
    pub fn from_id(_cls: &PyType, id: u8) -> PyResult<Self> {
        let cell = Cell::try_from_u8(id).map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok(Self(cell))
    }
    #[classmethod]
    pub fn from_str(cls: &PyType, s: String) -> PyResult<Self> {
        if s.len() != 1 {
            Err(pyo3::exceptions::PyValueError::new_err("Invalid string representation of cell."))
        } else {
            let cell = Cell::try_from_char(s.chars().next().unwrap()).map_err(pyo3::exceptions::PyValueError::new_err)?;
            Ok(Self(cell))
        }
    }
    #[getter]
    pub fn id(&self) -> PyResult<u8> { Ok(self.0.to_u8()) }
    pub fn __str__(&self) -> PyResult<String> {
        Ok(self.0.to_char().to_string())
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
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

#[derive(Clone)]
#[pyclass(name = "MoveDecisionResource")]
pub struct MoveDecisionResourceWrapper {
    resource: MoveDecisionResource,
}

#[pymethods]
impl MoveDecisionResourceWrapper {
    pub fn get_dst_candidates(&self) -> PyResult<HashSet<PlacementWrapper>> {
        let r = self.resource.dst_candidates.iter()
            .map(|&placement| PlacementWrapper { placement })
            .collect::<HashSet<_>>();
        Ok(r)
    }
}

#[pyclass(name = "Game")]
pub struct GameWrapper {
    game: Game<'static>,
}

#[pymethods]
impl GameWrapper {
    #[new]
    fn new() -> Self {
        Self { game: Default::default() }
    }
    pub fn fast_mode(&mut self) -> PyResult<()> {
        self.game.performance_mode();
        Ok(())
    }
    pub fn should_supply_next_pieces(&self) -> PyResult<bool> {
        Ok(self.game.should_supply_next_pieces())
    }
    pub fn supply_next_pieces(&mut self, piece_cell_ids: Vec<u8>) -> PyResult<()> {
        let mut pieces = Vec::with_capacity(piece_cell_ids.len());
        for cell_id in piece_cell_ids.iter() {
            let cell = Cell::try_from_u8(*cell_id).map_err(pyo3::exceptions::PyValueError::new_err)?;
            let p = cell.try_to_piece().map_err(pyo3::exceptions::PyValueError::new_err)?;
            pieces.push(p);
        }
        self.game.supply_next_pieces(&pieces);
        Ok(())
    }
    pub fn setup_falling_piece(&mut self, piece_cell_id: Option<u8>) -> PyResult<()> {
        if piece_cell_id.is_none() {
            return self.game.setup_falling_piece(None).map_err(pyo3::exceptions::PyRuntimeError::new_err);
        }
        let cell = Cell::try_from_u8(piece_cell_id.unwrap()).map_err(pyo3::exceptions::PyValueError::new_err)?;
        let p = cell.try_to_piece().map_err(pyo3::exceptions::PyValueError::new_err)?;
        self.game.setup_falling_piece(Some(p)).map_err(pyo3::exceptions::PyRuntimeError::new_err)
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
    pub fn can_hold(&self) -> PyResult<bool> {
        Ok(self.game.state.can_hold)
    }
    pub fn get_move_decision_resource(&self) -> PyResult<MoveDecisionResourceWrapper> {
        let resource = MoveDecisionResource::with_game(&self.game).map_err(pyo3::exceptions::PyRuntimeError::new_err)?;
        Ok(MoveDecisionResourceWrapper { resource })
    }
    pub fn set_falling_piece_placement(&mut self, dst: PlacementWrapper) -> PyResult<()> {
        if let Some(fp) = self.game.state.falling_piece.as_mut() {
            fp.placement = dst.placement;
        } else {
            return Err(pyo3::exceptions::PyRuntimeError::new_err("No falling piece."));
        }
        Ok(())
    }
    pub fn set_hold_piece(&mut self, piece_cell_id: Option<u8>) -> PyResult<()> {
        self.game.state.hold_piece = if let Some(cell_id) = piece_cell_id {
            let cell = Cell::try_from_u8(cell_id).map_err(pyo3::exceptions::PyValueError::new_err)?;
            let p = cell.try_to_piece().map_err(pyo3::exceptions::PyValueError::new_err)?;
            Some(p)
        } else {
            None
        };
        Ok(())
    }
    pub fn set_next_pieces(&mut self, next_piece_cell_ids: Vec<u8>) -> PyResult<()> {
        let mut pieces = VecDeque::new();
        for id in next_piece_cell_ids.iter() {
            let cell = Cell::try_from_u8(*id).map_err(pyo3::exceptions::PyValueError::new_err)?;
            let p = cell.try_to_piece().map_err(pyo3::exceptions::PyValueError::new_err)?;
            pieces.push_back(p);
        }
        self.game.state.next_pieces.pieces = pieces;
        Ok(())
    }
    pub fn set_playfield_with_u64_rows(&mut self, six_rows_x7: Vec<u64>) -> PyResult<()> {
        if six_rows_x7.len() != 7 {
            return Err(pyo3::exceptions::PyValueError::new_err("Invalid length of values."));
        }
        for (i, v) in six_rows_x7.iter().enumerate() {
            self.game.state.playfield.grid.set_rows_with_bits((0, i as Y * 6).into(), 10, *v);
        }
        Ok(())
    }
    pub fn get_playfield_as_u64_rows(&self) -> PyResult<Vec<u64>> {
        Ok(self.game.state.playfield.grid.bit_grid.to_int_values())
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
        g.set_playfield_with_u64_rows(vec![0b1100110011, 0, 0, 0, 0, 0, 0]).unwrap();
        println!("{}", g.__str__().unwrap());
    }
}
